use super::{Locked, size_of, spacecrab};
use std::{fs, io};
use tempfile::tempdir;

// a.txt is bigger than any folder's own size, so locked/ always sorts first.
const A_LEN: usize = 100_000;

#[test]
fn size_error_is_reported_and_scan_continues() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), vec![0; A_LEN])?;
    fs::create_dir(dir.join("locked"))?;
    fs::write(dir.join("locked/b.txt"), [0; 536])?;
    // Readable but not searchable: b.txt is listed, but its size can't be read.
    let _locked = Locked::new(dir.join("locked"), 0o444)?;

    let output = spacecrab(dir).arg("-b").output()?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    // locked/ only counts its own size.
    let (a, locked) = (size_of(dir.join("a.txt")), size_of(dir.join("locked")));
    let root = size_of(dir) + a + locked;
    assert_eq!(
        stdout,
        format!("{locked}\t./locked/\n{a}\t./a.txt\n{root}\t.\n")
    );
    assert!(stderr.starts_with("spacecrab: ./locked/b.txt: "));
    Ok(())
}

#[test]
fn child_that_cant_be_read_is_listed_with_nothing_counted() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::create_dir(dir.join("locked"))?;
    fs::write(dir.join("locked/a.txt"), [0; 1000])?;
    // Readable but not searchable, and this time it's PATH itself, so a.txt is a
    // direct child that's listed but can't be stat'ed.
    let _locked = Locked::new(dir.join("locked"), 0o444)?;

    let output = spacecrab(dir).args(["-b", "locked"]).output()?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let locked = size_of(dir.join("locked"));
    assert_eq!(stdout, format!("0\tlocked/a.txt\n{locked}\tlocked\n"));
    assert!(stderr.starts_with("spacecrab: locked/a.txt: "), "{stderr}");
    Ok(())
}

#[test]
fn unreadable_dir_is_reported_and_scan_continues() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), vec![0; A_LEN])?;
    fs::create_dir(dir.join("locked"))?;
    let _locked = Locked::new(dir.join("locked"), 0o000)?;

    let output = spacecrab(dir).arg("-b").output()?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let (a, locked) = (size_of(dir.join("a.txt")), size_of(dir.join("locked")));
    let root = size_of(dir) + a + locked;
    assert_eq!(
        stdout,
        format!("{locked}\t./locked/\n{a}\t./a.txt\n{root}\t.\n")
    );
    assert!(stderr.starts_with("spacecrab: ./locked: "));
    Ok(())
}
