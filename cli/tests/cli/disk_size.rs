use super::spacecrab;
use std::{fs, io};
use tempfile::tempdir;

#[cfg(unix)]
#[test]
fn sparse_file_counts_only_whats_on_disk() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::File::create(tmp.path().join("sparse"))?.set_len(1 << 30)?;

    let output = spacecrab(tmp.path()).args(["-b", "sparse"]).output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let (size, path) = stdout.trim_end().split_once('\t').unwrap();
    assert!(size.parse::<u64>().unwrap() < 1 << 20, "{stdout}");
    assert_eq!(path, "sparse");

    for flag in ["-A", "--apparent-size"] {
        let args = [flag, "-b", "sparse"];
        let output = spacecrab(tmp.path()).args(args).output()?;

        assert_eq!(output.status.code(), Some(0), "{flag}");
        assert_eq!(output.stdout, b"1073741824\tsparse\n", "{flag}");
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn written_file_counts_at_least_its_length() -> io::Result<()> {
    use std::io::Write as _;

    let tmp = tempdir()?;
    let mut file = fs::File::create(tmp.path().join("data"))?;
    file.write_all(&vec![1; 30_000_000])?;
    // Some filesystems only allocate blocks once the data is flushed.
    file.sync_all()?;

    let output = spacecrab(tmp.path()).args(["-b", "data"]).output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let (size, _) = stdout.split_once('\t').unwrap();
    assert!(size.parse::<u64>().unwrap() >= 30_000_000, "{stdout}");
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_default_is_file_length() -> io::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("a.txt"), [0; 1000])?;
    fs::create_dir(tmp.path().join("sub"))?;
    fs::write(tmp.path().join("sub").join("b.txt"), [0; 536])?;

    let default = spacecrab(tmp.path()).arg("-b").output()?;
    let apparent = spacecrab(tmp.path()).args(["-A", "-b"]).output()?;

    assert!(default.status.success());
    assert_eq!(default.stdout, apparent.stdout);
    Ok(())
}
