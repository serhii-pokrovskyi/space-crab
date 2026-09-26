use std::{fs, io, path::Path, process::Command};
use tempfile::tempdir;

#[test]
fn total_is_sum_of_listed_sizes() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::create_dir(dir.join("sub"))?;
    fs::write(dir.join("sub/b.txt"), [0; 536])?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(dir)
        .output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let (files, total) = stdout.split_once("\n\n").unwrap();
    let mut files: Vec<&str> = files.lines().collect();
    files.sort();

    let a = Path::new(".").join("a.txt");
    let b = Path::new(".").join("sub").join("b.txt");
    assert_eq!(
        files,
        [
            format!("{} 1000 B", a.display()),
            format!("{} 536 B", b.display())
        ]
    );
    assert_eq!(total, "total size: 1.50 KiB\n");
    Ok(())
}

#[cfg(unix)]
#[test]
fn size_error_is_reported_and_scan_continues() -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::create_dir(dir.join("locked"))?;
    fs::write(dir.join("locked/b.txt"), [0; 536])?;
    // Readable but not searchable: b.txt is listed, but its size can't be read.
    fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o444))?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(dir)
        .output();
    fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755))?;
    let output = output?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stdout, "./a.txt 1000 B\n\ntotal size: 1000 B\n");
    assert!(stderr.starts_with("spacecrab: ./locked/b.txt: "));
    Ok(())
}

#[cfg(unix)]
#[test]
fn closed_stdout_exits_cleanly() -> io::Result<()> {
    use std::process::Stdio;

    let tmp = tempdir()?;

    let mut child = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
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
    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .stdout(full)
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("spacecrab: error writing output: "),
        "{stderr}"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn closed_stderr_does_not_panic() -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Stdio;

    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::create_dir(dir.join("locked"))?;
    fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000))?;

    let child = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let output = child.and_then(|mut child| {
        // Close the read end before the CLI reports the error, like `spacecrab 2>&1 | head -0`.
        drop(child.stderr.take());
        child.wait_with_output()
    });
    fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755))?;
    let output = output?;

    // 1 because the scan was incomplete; a panic would exit with 101.
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

#[cfg(unix)]
#[test]
fn unreadable_dir_is_reported_and_scan_continues() -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::create_dir(dir.join("locked"))?;
    fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000))?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(dir)
        .output();
    fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755))?;
    let output = output?;

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stdout, "./a.txt 1000 B\n\ntotal size: 1000 B\n");
    assert!(stderr.starts_with("spacecrab: ./locked: "));
    Ok(())
}

#[test]
fn path_argument_is_scanned() -> io::Result<()> {
    let tmp = tempdir()?;
    let target = tmp.path().join("target");
    fs::create_dir(&target)?;
    fs::write(target.join("a.txt"), [0; 1000])?;
    fs::write(tmp.path().join("outside.txt"), [0; 5])?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .arg(&target)
        .output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let a = target.join("a.txt");
    assert_eq!(
        stdout,
        format!("{} 1000 B\n\ntotal size: 1000 B\n", a.display())
    );
    Ok(())
}

#[test]
fn usage_error_exits_2_without_scanning() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;

    for args in [&["--bogus"][..], &["-q"], &["a", "b"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
            .current_dir(tmp.path())
            .args(args)
            .output()?;

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(output.stdout, b"", "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("error: "), "{args:?}: {stderr}");
        assert!(stderr.contains("Usage: spacecrab"), "{args:?}: {stderr}");
    }
    Ok(())
}

#[test]
fn double_dash_ends_options() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::create_dir(tmp.path().join("-n"))?;
    fs::write(tmp.path().join("-n").join("a.txt"), [0; 1000])?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .args(["--", "-n"])
        .output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let a = Path::new("-n").join("a.txt");
    assert_eq!(
        stdout,
        format!("{} 1000 B\n\ntotal size: 1000 B\n", a.display())
    );
    Ok(())
}

#[test]
fn missing_path_exits_1_without_output() -> io::Result<()> {
    let tmp = tempdir()?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .arg("missing")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("spacecrab: missing: "), "{stderr}");
    Ok(())
}

#[test]
fn file_path_exits_1_without_output() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .arg("a.txt")
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("spacecrab: a.txt: "), "{stderr}");
    Ok(())
}

#[cfg(unix)]
#[test]
fn unreadable_root_exits_1_without_output() -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempdir()?;
    fs::create_dir(tmp.path().join("locked"))?;
    fs::set_permissions(tmp.path().join("locked"), fs::Permissions::from_mode(0o000))?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .arg("locked")
        .output();
    fs::set_permissions(tmp.path().join("locked"), fs::Permissions::from_mode(0o755))?;
    let output = output?;

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("spacecrab: locked: "), "{stderr}");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_path_to_dir_is_scanned() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::create_dir(tmp.path().join("real"))?;
    fs::write(tmp.path().join("real/a.txt"), [0; 1000])?;
    std::os::unix::fs::symlink("real", tmp.path().join("link"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_spacecrab"))
        .current_dir(tmp.path())
        .arg("link")
        .output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "link/a.txt 1000 B\n\ntotal size: 1000 B\n");
    Ok(())
}
