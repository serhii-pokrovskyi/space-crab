use super::{Locked, spacecrab};
use std::{fs, io, process::Stdio};
use tempfile::tempdir;

#[test]
fn closed_stdout_exits_cleanly() -> io::Result<()> {
    let tmp = tempdir()?;

    let mut child = spacecrab(tmp.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Close the read end before the CLI prints anything, like `spacecrab | head -0`.
    drop(child.stdout.take());
    let output = child.wait_with_output()?;

    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert!(output.status.success());
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn output_write_error_is_reported() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;

    // Every write to /dev/full fails with ENOSPC, like a full disk.
    let full = fs::OpenOptions::new().write(true).open("/dev/full")?;
    let output = spacecrab(tmp.path()).stdout(full).output()?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("spacecrab: error writing output: "),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn closed_stderr_does_not_panic() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::create_dir(dir.join("locked"))?;
    let _locked = Locked::new(dir.join("locked"), 0o000)?;

    let mut child = spacecrab(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Close the read end before the CLI reports the error, like `spacecrab 2>&1 | head -0`.
    drop(child.stderr.take());
    let output = child.wait_with_output()?;

    // 1 because the scan was incomplete; a panic would exit with 101.
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}
