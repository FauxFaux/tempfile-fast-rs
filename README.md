## (Slightly) faster temporary files on Linux

[![Build status](https://api.travis-ci.org/FauxFaux/tempfile-fast-rs.png)](https://travis-ci.org/FauxFaux/tempfile-fast-rs)
[![](https://img.shields.io/crates/v/temfile-fast.svg)](https://crates.io/crates/tempfile-fast)

On "recent" Linux (~2014+), filesystems support a new type of unnamed, persistable temporary file.

The excellent [`tempfile`](https://crates.io/crates/tempfile) crate
  supports two types of temporary files:

 * unnamed temporary files, which are secure and fast, but which 
    can never be written into real the filesystem.
 * named temporary files, which have some theoretical security risks and
    performance problems, but you must use if you ever want to write
    the data into the real filesystem efficiently.

It does not, however, expose unnamed (secure, fast), persistable (convenient) files.
This crate does.

On non-modern-Linux, this crate falls back to using `tempfile`'s `NamedTemporaryFile` directly.


### "recent" Linux

Support for `O_TMPFILE` was added to:

 * Linux 3.11: ext2, ext3, ext4, UDF, Minix, shmem.
 * Linux 3.15: xfs
 * Linux 3.16: btrfs, f2fs
 * Linux 4.9: ubifs  

Some distros, with release dates, End of Life dates, and kernel versions:

 * Ubuntu 12.04 (Apr 2012 - Apr 2017): 3.2
 * Debian Wheezy (May 2013 - May 2018): 3.2
 * Ubuntu 14.04.0 (Apr 2014 - Apr 2019): 3.13
 * RHEL 7 (Jun 2014 - Jun 2024): 3.10
 * Debian Jessie (Apr 2015 - Apr 2020): 3.16
 * OpenSUSE Leap 42.1 (Nov 2015 - May 2017): 4.1
 * Ubuntu 14.04.5 (Aug 2016 - Apr 2019): 4.4
 * Debian Stretch (Jun 2017 - ): 4.9

i.e. as of 2017, the latest version, and even the previous version,
of everything except CentOS/RHEL looks pretty promising. 


### Upstreaming

An alternative implementation, ramming this into `NamedTemporaryFile`,
was discussed in [a pull-request](https://github.com/Stebalien/tempfile/pull/31).
This was the result.
