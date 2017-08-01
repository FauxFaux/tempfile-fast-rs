extern crate tempdir;
extern crate tempfile_fast;

use std::fs;

use tempfile_fast::persistable_tempfile_in;

#[test]
fn empty_on_linux() {
    let temp_dir = tempdir::TempDir::new("tempfile-deleted").unwrap();
    let tmp = persistable_tempfile_in(&temp_dir).unwrap();

    // Will only actually be deleted on (modern) linux:
    #[cfg(target_os = "linux")] {
        assert_eq!(0, fs::read_dir(&temp_dir).unwrap().count());
    }

    let dest = temp_dir.path().to_path_buf().join("foo");

    tmp.persist_noclobber(&dest).unwrap();
    assert!(dest.exists());
}
