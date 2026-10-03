use super::spacecrab;
use std::{fs, io};
use tempfile::tempdir;

#[test]
fn help_is_printed_without_scanning() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;

    for flag in ["--help", "-h"] {
        let output = spacecrab(tmp.path()).arg(flag).output()?;

        assert_eq!(output.status.code(), Some(0), "{flag}");
        assert_eq!(output.stderr, b"", "{flag}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Usage: spacecrab"), "{flag}: {stdout}");
        assert!(!stdout.contains("a.txt"), "{flag}: {stdout}");
    }
    Ok(())
}

#[test]
fn help_explains_sizes_and_exit_codes() -> io::Result<()> {
    let tmp = tempdir()?;

    let output = spacecrab(tmp.path()).arg("--help").output()?;

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Sizes are space used on disk"), "{stdout}");
    assert!(
        stdout.contains("On Windows sizes are always file lengths."),
        "{stdout}"
    );
    assert!(stdout.contains("  0  complete\n"), "{stdout}");
    assert!(
        stdout.contains("  1  results incomplete, or the report couldn't be written\n"),
        "{stdout}"
    );
    assert!(stdout.contains("  2  usage error\n"), "{stdout}");
    Ok(())
}

#[test]
fn version_is_printed() -> io::Result<()> {
    let tmp = tempdir()?;

    for flag in ["--version", "-V"] {
        let output = spacecrab(tmp.path()).arg(flag).output()?;

        assert_eq!(output.status.code(), Some(0), "{flag}");
        assert_eq!(output.stderr, b"", "{flag}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout, format!("spacecrab {}\n", env!("CARGO_PKG_VERSION")));
    }
    Ok(())
}

#[test]
fn help_and_version_win_over_path() -> io::Result<()> {
    let tmp = tempdir()?;

    for flag in ["--help", "--version"] {
        let output = spacecrab(tmp.path()).args(["missing", flag]).output()?;

        assert_eq!(output.status.code(), Some(0), "{flag}");
        assert_eq!(output.stderr, b"", "{flag}");
        assert!(!output.stdout.is_empty(), "{flag}");
    }
    Ok(())
}
