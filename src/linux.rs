use std::fs;
use std::io;

use std::path::Path;
use std::ffi::CString;

use std::os::unix::io::{FromRawFd, AsRawFd};
use std::os::unix::ffi::OsStrExt;

use libc::open64 as open;
use libc::{linkat, c_char, c_int, O_RDWR, O_CLOEXEC};
const O_TMPFILE: c_int = 0o20200000;

pub fn create_nonexclusive_tempfile_in<P: AsRef<Path>>(dir: P) -> io::Result<fs::File> {
    create(dir.as_ref())
}

pub fn link_at<P: AsRef<Path>>(what: fs::File, dest: P) -> io::Result<()> {
    let old_path: CString = CString::new(format!("/proc/self/fd/{}", what.as_raw_fd())).unwrap();
    let new_path = cstr(dest.as_ref())?;

    unsafe { link_symlink_fd_at(&old_path, &new_path) }
}



// Stolen from tempfile / std < 1.6.0.
pub fn cstr(path: &Path) -> io::Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contained a null"))
}

pub fn create(dir: &Path) -> io::Result<fs::File> {
    match unsafe {
        let path = cstr(dir)?;
        open(path.as_ptr(),
             O_CLOEXEC | O_TMPFILE | O_RDWR,
             0o600)
    } {
        -1 => Err(io::ErrorKind::InvalidInput.into()),
        fd => Ok(unsafe { FromRawFd::from_raw_fd(fd) }),
    }
}

/// Attempt to link an old symlink to a file back into the filesystem.
unsafe fn link_symlink_fd_at(old_path: &CString, new_path: &CString) -> io::Result<()> {
    const AT_FDCWD: c_int = -100;
    const AT_SYMLINK_FOLLOW: c_int = 0x400;

    if linkat(AT_FDCWD,
              old_path.as_ptr() as *const c_char,
              AT_FDCWD,
              new_path.as_ptr() as *const c_char,
              AT_SYMLINK_FOLLOW) != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}