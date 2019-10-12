//! # Sponge
//!
//! A [`Sponge`] is a safe and efficient way to update a file in place.
//!
//! ### Example
//!
//! ```rust
//! # use std::io::Write;
//! let mut temp = tempfile_fast::Sponge::new_for("example.txt").unwrap();
//! temp.write_all(b"hello").unwrap();
//! temp.commit().unwrap();
//! ```
//!
//! ## PersistableTempFile
//!
//! The raw [`PersistableTempFile`] is also available. However,
//! You probably want to use the `tempfile` crate unless you have
//! hit a known performance problem, or you only care about modern
//! Linux. See `README.md` for more details.
//!
//! ### Example (raw)
//!
//! ```rust,no_run
//! # use std::io::Write;
//! let mut temp = tempfile_fast::PersistableTempFile::new_in("/var/lib/foo").unwrap();
//! temp.write_all(b"hello").unwrap();
//! temp.persist_noclobber("/var/lib/foo/bar").unwrap();
//! ```

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod linux {
    use std::fs;
    use std::io;
    use std::path::Path;

    #[inline]
    pub fn create_nonexclusive_tempfile_in<P>(_dir: P) -> io::Result<fs::File> {
        Err(io::ErrorKind::InvalidInput.into())
    }

    #[inline]
    pub fn link_at<P: AsRef<Path>>(_what: &fs::File, _dest: P) -> io::Result<()> {
        Err(io::ErrorKind::InvalidData.into())
    }
}

mod persistable;
mod sponge;

pub use crate::persistable::PersistError;
pub use crate::persistable::PersistableTempFile;
pub use sponge::Sponge;
