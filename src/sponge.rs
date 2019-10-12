use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::PersistableTempFile;

/// A safer abstraction for atomic overwrites of files.
///
/// A `Sponge` will "soak up" writes, and eventually, when you're ready, write them to the destination file.
/// This is atomic, so the destination file will never be left in an intermediate state. This is
/// error, panic, and crash safe.
///
/// Ownership and permission is preserved, where appropriate for the platform.
///
/// Space is needed to soak up these writes: if you are overwriting a large file, you may need
/// disc space for the entire file to be stored twice.
///
/// For performance and correctness reasons, many of the things that can go wrong will go wrong at
/// `commit()` time, not on creation. This might not be what you want if you are doing a very
/// expensive operation. Most of the failures are permissions errors, however. If you are operating
/// as a single user inside the user's directory, the chance of failure (except for disc space) is
/// negligible.
///
/// # Example
///
/// ```rust
/// # use std::io::Write;
/// let mut temp = tempfile_fast::Sponge::new_for("example.txt").unwrap();
/// temp.write_all(b"hello").unwrap();
/// temp.commit().unwrap();
/// ```
pub struct Sponge {
    dest: PathBuf,
    temp: io::BufWriter<PersistableTempFile>,
}

impl Sponge {
    /// Create a `Sponge` which will eventually overwrite the named file.
    /// The file does not have to exist.
    ///
    /// This will be resolved to an absolute path relative to the current directory immediately.
    ///
    /// The path is *not* run through [`fs::canonicalize`], so other oddities will resolve
    /// at `commit()` time. Notably, a `symlink` (or `hardlink`, or `reflink`) will be converted
    /// into a regular file, using the target's [`fs::metadata`].
    ///
    /// Intermediate directories will be created using the platform defaults (e.g. permissions),
    /// if this is not what you want, create them in advance.
    pub fn new_for<P: AsRef<Path>>(path: P) -> Result<Sponge, io::Error> {
        let path = path.as_ref();

        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let mut absolute = env::current_dir()?;
            absolute.push(path);
            absolute
        };

        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "path must have a parent"))?;

        fs::create_dir_all(parent)?;

        Ok(Sponge {
            temp: io::BufWriter::new(PersistableTempFile::new_in(parent)?),
            dest: path,
        })
    }

    /// Write the `Sponge` out to the destination file.
    ///
    /// Ownership and permission is preserved, where appropriate for the platform. The permissions
    /// and ownership are resolved now, using the (absolute) path provided. i.e. changes to the
    /// destination's file's permissions since the creation of the `Sponge` will be included.
    ///
    /// The aim is to transfer all ownership and permission information, but not timestamps.
    /// The implementation, and what information is transferred, is subject to change in minor
    /// versions.
    ///
    /// The file is `flush()`ed correctly, but not `fsync()`'d. The update is atomic against
    /// anything that happens to the current process, including erroring, panicking, or crashing.
    ///
    /// If you need the update to survive power loss, or OS/kernel issues, you should additionally
    /// follow the platform recommendations for `fsync()`, which may involve calling `fsync()` on
    /// at least the new file, and probably on the parent directory. Note that this is the same as
    /// every other file API, but is being called out here as a reminder, if you are building
    /// certain types of application.
    ///
    /// ## Platform-specific behavior
    ///
    /// Metadata:
    /// * `unix` (including `linux`): At least `chown(uid, gid)` and `chmod(mode_t)`
    /// * `windows`: At least the `readonly` flag.
    /// * all: See [`fs::set_permissions`]
    ///
    /// ## Error
    ///
    /// If any underlying operation fails the system error will be returned directly. This method
    /// consumes `self`, so these errors are not recoverable. Failing to set the ownership
    /// information on the temporary file is an error, not ignored, unlike in many implementations.
    pub fn commit(self) -> Result<(), io::Error> {
        let temp = self.temp.into_inner()?;
        copy_metadata(&self.dest, temp.as_ref())?;
        temp.persist_by_rename(self.dest)
            .map_err(|persist_error| persist_error.error)?;
        Ok(())
    }
}

/// A `Sponge` is a `BufWriter`.
impl io::Write for Sponge {
    /// `write` to the intermediate file, without touching the destination.
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.temp.write(buf)
    }

    /// `flush` to the intermediate file, without touching the destination.
    /// This has no real purpose, as these writes should not be observable.
    fn flush(&mut self) -> Result<(), io::Error> {
        self.temp.flush()
    }
}

fn copy_metadata(source: &Path, dest: &fs::File) -> Result<(), io::Error> {
    let metadata = match source.metadata() {
        Ok(metadata) => metadata,
        Err(ref e) if io::ErrorKind::NotFound == e.kind() => {
            return Ok(());
        }
        Err(e) => Err(e)?,
    };

    dest.set_permissions(metadata.permissions())?;

    #[cfg(unix)]
    unix_chown::chown(metadata, dest)?;

    Ok(())
}

#[cfg(unix)]
mod unix_chown {
    use std::fs;
    use std::io;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::io::AsRawFd;

    pub fn chown(source: fs::Metadata, dest: &fs::File) -> Result<(), io::Error> {
        let fd = dest.as_raw_fd();
        zero_success(unsafe { libc::fchown(fd, source.uid(), source.gid()) })?;
        Ok(())
    }

    fn zero_success(err: libc::c_int) -> Result<(), io::Error> {
        if 0 == err {
            return Ok(());
        }

        Err(io::Error::last_os_error())
    }
}
