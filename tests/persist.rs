extern crate tempdir;
extern crate tempfile_fast;

use std::fs;
use std::io::Read;
use std::io::Write;

use tempfile_fast::PersistableTempFile;

#[test]
fn empty_on_linux() {
    let temp_dir = tempdir::TempDir::new("tempfile-deleted").unwrap();
    let tmp = PersistableTempFile::new_in(&temp_dir).unwrap();

    // Will only actually be deleted on (modern) linux:
    #[cfg(target_os = "linux")]
    {
        assert_eq!(0, fs::read_dir(&temp_dir).unwrap().count());
    }

    let dest = temp_dir.path().to_path_buf().join("foo");

    tmp.persist_noclobber(&dest).unwrap();
    assert!(dest.exists());
}

#[test]
fn overwrite() {
    let temp_dir = tempdir::TempDir::new("tempfile-deleted").unwrap();
    let root = temp_dir.path();
    let mut sub = root.to_path_buf();
    sub.push("sub");
    fs::create_dir(&sub).unwrap();

    let mut dest = sub.to_path_buf();
    dest.push("dest");
    {
        fs::File::create(&dest).unwrap();
    }

    let tmp = PersistableTempFile::new_in(&sub).unwrap();
    let mut tmp = match tmp.persist_noclobber(&dest) {
        Ok(()) => unreachable!(),
        Err(e) => e.file,
    };

    tmp.write_all(b"yello").unwrap();

    assert!(tmp.persist_by_rename(&dest).is_ok());

    {
        let mut s = String::new();
        fs::File::open(&dest)
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert_eq!(s, "yello");
    }

    assert_eq!(1, fs::read_dir(sub).unwrap().count());
    assert_eq!(
        1,
        fs::read_dir(root).unwrap().count(),
        "didn't create anything in the parent directory, either"
    );
}
