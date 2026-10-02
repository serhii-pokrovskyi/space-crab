use super::{Locked, spacecrab};
use std::{fs, io};
use tempfile::tempdir;

#[test]
fn size_error_is_reported_and_scan_continues() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::create_dir(dir.join("locked"))?;
    fs::write(dir.join("locked/b.txt"), [0; 536])?;
    // Readable but not searchable: b.txt is listed, but its size can't be read.
    let _locked = Locked::new(dir.join("locked"), 0o444)?;

    let output = spacecrab(dir).output()?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stdout, "     1000 B\t./a.txt\n     1000 B\t.\n");
    assert!(stderr.starts_with("spacecrab: ./locked/b.txt: "));
    Ok(())
}

#[test]
fn unreadable_dir_is_reported_and_scan_continues() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::create_dir(dir.join("locked"))?;
    let _locked = Locked::new(dir.join("locked"), 0o000)?;

    let output = spacecrab(dir).output()?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stdout, "     1000 B\t./a.txt\n     1000 B\t.\n");
    assert!(stderr.starts_with("spacecrab: ./locked: "));
    Ok(())
}
