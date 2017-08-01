use std::fs;
use std::io;
use std::path::Path;

use tempfile;
use linux;

pub enum PersistableTempFile {
    Linux(fs::File),
    Fallback(tempfile::NamedTempFile),
}

use self::PersistableTempFile::*;

pub fn persistable_tempfile_in<P: AsRef<Path>>(dir: P) -> io::Result<PersistableTempFile> {
    if let Ok(file) = linux::create_nonexclusive_tempfile_in(&dir) {
        return Ok(Linux(file));
    }

    Ok(Fallback(tempfile::NamedTempFileOptions::new().create_in(dir)?))
}

impl AsRef<fs::File> for PersistableTempFile {
    fn as_ref(&self) -> &fs::File {
        match self {
            &Linux(ref file) => file,
            &Fallback(ref named) => named,
        }
    }
}

impl PersistableTempFile {
    pub fn persist_noclobber<P: AsRef<Path>>(self, dest: P) -> io::Result<()> {
        match self {
            Linux(file) => linux::link_at(file, dest),
            Fallback(named) => {
                named.persist_noclobber(dest)?;
                Ok(())
            },
        }
    }
}