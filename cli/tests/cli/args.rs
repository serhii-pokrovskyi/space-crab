use super::{size_of, spacecrab};
use std::{fs, io, path::Path};
use tempfile::tempdir;

#[test]
fn path_argument_is_scanned() -> io::Result<()> {
    let tmp = tempdir()?;
    let target = tmp.path().join("target");
    fs::create_dir(&target)?;
    fs::write(target.join("a.txt"), [0; 1000])?;
    fs::write(tmp.path().join("outside.txt"), [0; 5])?;

    let output = spacecrab(tmp.path()).arg("-b").arg(&target).output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let a = target.join("a.txt");
    let a_size = size_of(&a);
    assert_eq!(
        stdout,
        format!(
            "{a_size}\t{}\n{}\t{}\n",
            a.display(),
            size_of(&target) + a_size,
            target.display()
        )
    );
    Ok(())
}

#[test]
fn usage_error_exits_2_without_scanning() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;

    for args in [&["--bogus"][..], &["-q"], &["a", "b"]] {
        let output = spacecrab(tmp.path()).args(args).output()?;

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

    let output = spacecrab(tmp.path()).args(["-b", "--", "-n"]).output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let a = Path::new("-n").join("a.txt");
    let a_size = size_of(tmp.path().join(&a));
    let n_size = size_of(tmp.path().join("-n"));
    assert_eq!(
        stdout,
        format!("{a_size}\t{}\n{}\t-n\n", a.display(), n_size + a_size)
    );
    Ok(())
}

#[test]
fn missing_path_exits_1_without_output() -> io::Result<()> {
    let tmp = tempdir()?;

    let output = spacecrab(tmp.path()).arg("missing").output()?;

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("spacecrab: missing: "), "{stderr}");
    Ok(())
}

#[test]
fn file_path_prints_only_its_own_line() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;

    let output = spacecrab(tmp.path()).args(["-b", "a.txt"]).output()?;

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, b"");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let size = size_of(tmp.path().join("a.txt"));
    assert_eq!(stdout, format!("{size}\ta.txt\n"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn unreadable_root_exits_1_without_output() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::create_dir(tmp.path().join("locked"))?;
    let _locked = super::Locked::new(tmp.path().join("locked"), 0o000)?;

    let output = spacecrab(tmp.path()).arg("locked").output()?;

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

    let output = spacecrab(tmp.path()).args(["-b", "link"]).output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    // The root symlink is followed, so the root is the real folder's size.
    let real = tmp.path().join("real");
    let a_size = size_of(real.join("a.txt"));
    let total = size_of(&real) + a_size;
    assert_eq!(stdout, format!("{a_size}\tlink/a.txt\n{total}\tlink\n"));
    Ok(())
}
