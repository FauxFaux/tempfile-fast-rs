use std::fmt;
use std::fs;
use std::io;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

use tempfile;
use linux;

pub enum PersistableTempFile {
    Linux(fs::File),
    Fallback(tempfile::NamedTempFile),
}

use self::PersistableTempFile::*;

impl PersistableTempFile {
    /// Create a temporary file in a given filesystem, or, if the filesystem does not support
    /// creating secure temporary files, create a `tempfile::NamedTemporaryFile`.
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<PersistableTempFile> {
        if let Ok(file) = linux::create_nonexclusive_tempfile_in(&dir) {
            return Ok(Linux(file));
        }

        Ok(Fallback(tempfile::NamedTempFileOptions::new()
            .create_in(dir)?))
    }
}

impl AsRef<fs::File> for PersistableTempFile {
    fn as_ref(&self) -> &fs::File {
        match *self {
            Linux(ref file) => file,
            Fallback(ref named) => named,
        }
    }
}

impl AsMut<fs::File> for PersistableTempFile {
    fn as_mut(&mut self) -> &mut fs::File {
        match *self {
            Linux(ref mut file) => file,
            Fallback(ref mut named) => named,
        }
    }
}

impl Deref for PersistableTempFile {
    type Target = fs::File;
    #[inline]
    fn deref(&self) -> &fs::File {
        self.as_ref()
    }
}

impl DerefMut for PersistableTempFile {
    #[inline]
    fn deref_mut(&mut self) -> &mut fs::File {
        self.as_mut()
    }
}

impl fmt::Debug for PersistableTempFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PersistableTempFile::{}",
            match *self {
                Linux(_) => "Linux",
                Fallback(_) => "Fallback",
            }
        )
    }
}

/// Error returned when persisting a temporary file fails
#[derive(Debug)]
pub struct PersistError {
    /// The underlying IO error.
    pub error: io::Error,
    /// The temporary file that couldn't be persisted.
    pub file: PersistableTempFile,
}

impl PersistableTempFile {
    /// Store this temporary file into a real file path.
    ///
    /// The path must not exist, and must be on the same "filesystem".
    pub fn persist_noclobber<P: AsRef<Path>>(self, dest: P) -> Result<(), PersistError> {
        match self {
            Linux(file) => linux::link_at(&file, &dest).map_err(|error| PersistError {
                error,
                file: PersistableTempFile::Linux(file),
            }),
            Fallback(named) => named.persist_noclobber(dest).map(|_| ()).map_err(|e| {
                PersistError {
                    error: e.error,
                    file: PersistableTempFile::Fallback(e.file),
                }
            }),
        }
    }
}
