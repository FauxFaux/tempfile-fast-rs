use std::fmt;
use std::fs;
use std::io;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

use rand::Rng;

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

        Ok(Fallback(tempfile::NamedTempFileOptions::new().create_in(dir)?))
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

impl From<tempfile::PersistError> for PersistError {
    fn from(e: tempfile::PersistError) -> Self {
        PersistError {
            error: e.error,
            file: PersistableTempFile::Fallback(e.file),
        }
    }
}

impl PersistableTempFile {
    /// Store this temporary file into a real file path.
    ///
    /// The path must not exist, and must be on the same "filesystem".
    pub fn persist_noclobber<P: AsRef<Path>>(self, dest: P) -> Result<(), PersistError> {
        match self {
            Linux(file) => {
                Self::persist_noclobber_file(&file, dest).map_err(|error| PersistError {
                    error,
                    file: PersistableTempFile::Linux(file),
                })
            }
            Fallback(named) => named
                .persist_noclobber(dest)
                .map(|_| ())
                .map_err(PersistError::from),
        }
    }

    fn persist_noclobber_file<P: AsRef<Path>>(file: &fs::File, dest: P) -> io::Result<()> {
        linux::link_at(file, dest)
    }

    pub fn persist_by_rename<P: AsRef<Path>>(self, dest: P) -> Result<(), PersistError> {
        let file = match self {
            Linux(file) => file,
            Fallback(named) => return named.persist(dest).map(|_| ()).map_err(PersistError::from),
        };

        if Self::persist_noclobber_file(&file, &dest).is_ok() {
            return Ok(());
        };

        let mut dest_tmp = dest.as_ref().to_path_buf();
        let mut rng = ::rand::thread_rng();

        // pop the filename off
        dest_tmp.pop();

        for _ in 0..32768 {
            // add a new filename
            dest_tmp.push(format!(
                ".{}.tmp",
                rng.gen_ascii_chars().take(6).collect::<String>()
            ));

            match Self::persist_noclobber_file(&file, &dest_tmp) {
                Ok(()) => {
                    // we succeeded in converting into a named temporary file,
                    // now overwrite the destination
                    return match fs::rename(&dest_tmp, dest) {
                        Ok(()) => Ok(()),
                        Err(error) => {
                            // we couldn't overwrite the destination. Try and remove the
                            // temporary file we created, but, if we can't, just sigh.
                            let _ = fs::remove_file(&dest_tmp);

                            Err(PersistError {
                                error,
                                file: PersistableTempFile::Linux(file),
                            })
                        }
                    };
                }
                Err(error) => if io::ErrorKind::AlreadyExists != error.kind() {
                    return Err(PersistError {
                        error,
                        file: PersistableTempFile::Linux(file),
                    });
                },
            };
            dest_tmp.pop();
        }

        Err(PersistError {
            error: io::Error::new(io::ErrorKind::Other, "couldn't create temporary file"),
            file: PersistableTempFile::Linux(file),
        })
    }
}
