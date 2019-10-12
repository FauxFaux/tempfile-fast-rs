use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;

#[test]
fn sponge() -> Result<(), io::Error> {
    let dir = tempfile::TempDir::new()?;
    let mut test_path = dir.path().to_path_buf();
    {
        test_path.push("hello");
        fs::create_dir_all(&test_path)?;
        test_path.push("world.txt");
        fs::File::create(&test_path)?.write_all(b"content before")?;
    }

    let mut sponge = tempfile_fast::Sponge::new_for(&test_path)?;

    sponge.write_all(b"new stuff")?;
    assert_eq!("content before", read(fs::File::open(&test_path)?));

    sponge.flush()?;
    assert_eq!("content before", read(fs::File::open(&test_path)?));

    sponge.commit()?;
    assert_eq!("new stuff", read(fs::File::open(&test_path)?));

    assert_eq!(1, fs::read_dir(test_path.parent().unwrap())?.count());

    Ok(())
}

fn read<R: Read>(mut thing: R) -> String {
    let mut s = String::new();
    thing.read_to_string(&mut s).unwrap();
    s
}
