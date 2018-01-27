use std::fmt;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;

use rand::Rng;

use tempfile;
use linux;

/// An abstraction over different platform-specific temporary file optimisations.
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
    #[inline]
    fn as_ref(&self) -> &fs::File {
        match *self {
            Linux(ref file) => file,
            Fallback(ref named) => named,
        }
    }
}

impl AsMut<fs::File> for PersistableTempFile {
    #[inline]
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

impl Read for PersistableTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_mut().read(buf)
    }
}

impl Write for PersistableTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.as_mut().flush()
    }
}

impl Seek for PersistableTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.as_mut().seek(pos)
    }
}

impl<'a> Read for &'a PersistableTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_ref().read(buf)
    }
}

impl<'a> Write for &'a PersistableTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_ref().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.as_ref().flush()
    }
}

impl<'a> Seek for &'a PersistableTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.as_ref().seek(pos)
    }
}

#[cfg(unix)]
impl ::std::os::unix::io::AsRawFd for PersistableTempFile {
    #[inline]
    fn as_raw_fd(&self) -> ::std::os::unix::io::RawFd {
        self.as_ref().as_raw_fd()
    }
}

#[cfg(windows)]
impl ::std::os::windows::io::AsRawHandle for PersistableTempFile {
    #[inline]
    fn as_raw_handle(&self) -> ::std::os::windows::io::RawHandle {
        self.as_ref().as_raw_handle()
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

    /// Store this temporary file into a real name.
    ///
    /// The path must be on the same filesystem. It may exist, and will be overwritten.
    ///
    /// This method may create a named temporary file, and, in pathological failure cases,
    /// may silently fail to remove this temporary file. Sorry.
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
            dest_tmp.push(format!(".{:x}.tmp", rng.next_u64()));

            match Self::persist_noclobber_file(&file, &dest_tmp) {
                Ok(()) => {
                    // we succeeded in converting into a named temporary file,
                    // now overwrite the destination
                    return fs::rename(&dest_tmp, dest).map_err(|error| {
                        // we couldn't overwrite the destination. Try and remove the
                        // temporary file we created, but, if we can't, just sigh.
                        let _ = fs::remove_file(&dest_tmp);

                        PersistError {
                            error,
                            file: PersistableTempFile::Linux(file),
                        }
                    });
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
