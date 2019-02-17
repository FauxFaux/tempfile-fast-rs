//! You probably want to use the `tempfile` crate unless you have
//! hit a known performance problem, or you only care about modern
//! Linux. See `README.md` for more details.
//!
//! # Example
//!
//! ```rust,no_run
//! # use std::io::Write;
//! extern crate tempfile_fast;
//! let mut temp =  tempfile_fast::PersistableTempFile::new_in("/var/lib/foo").unwrap();
//! temp.write_all(b"hello").unwrap();
//! temp.persist_noclobber("/var/lib/foo/bar").unwrap();
//! ```

extern crate rand;
extern crate tempfile;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod linux {
    use std::fs;
    use std::io;
    use std::path::Path;

    #[inline]
    fn create_nonexclusive_tempfile_in<P>(dir: P) -> io::Result<fs::File> {
        Err(io::ErrorKind::InvalidInput.into())
    }

    #[inline]
    fn link_at<P: AsRef<Path>>(what: &fs::File, dest: P) -> io::Result<()> {
        Err(io::ErrorKind::InvalidInput.into())
    }
}

mod persistable;

pub use persistable::PersistableTempFile;
