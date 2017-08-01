extern crate tempfile;
extern crate libc;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(target_os = "linux"))]
mod linux {
    use std::fs;
    use std::io;

    fn create_nonexclusive_tempfile_in<P>(dir: P) -> io::Result<fs::File> {
        Err(io::ErrorKind::InvalidInput.into())
    }
}

mod persistable;

pub use persistable::persistable_tempfile_in;
