use super::spacecrab;
use std::{
    fs,
    io::{self, Write},
    os::unix::net::UnixListener,
    path::Path,
    process::Command,
};
use tempfile::{TempDir, tempdir_in};

fn write_synced(path: &Path, data: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    // Block counts can lag behind the write until the data is flushed.
    file.sync_all()
}

// Everything that tends to trip up a disk usage count. Lives in the target
// dir, not /tmp, which is often tmpfs. The listener is returned too, so it's a
// live socket while both tools look at it.
fn fixture() -> io::Result<(TempDir, UnixListener)> {
    let tmp = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
    let dir = tmp.path();

    let sparse = fs::File::create(dir.join("sparse"))?;
    sparse.set_len(1 << 30)?;
    sparse.sync_all()?;

    fs::create_dir_all(dir.join("nested/deeper"))?;
    write_synced(&dir.join("big"), &vec![1; 10 << 20])?;
    fs::hard_link(dir.join("big"), dir.join("big-link"))?;
    fs::hard_link(dir.join("big"), dir.join("nested/deeper/big-link"))?;
    write_synced(&dir.join("nested/deeper/file"), &[1; 5000])?;

    fs::create_dir(dir.join("many"))?;
    for i in 0..1000 {
        write_synced(&dir.join("many").join(i.to_string()), b"x")?;
    }

    std::os::unix::fs::symlink("big", dir.join("link"))?;
    let status = Command::new("mkfifo").arg(dir.join("fifo")).status()?;
    assert!(status.success());
    let socket = UnixListener::bind(dir.join("socket"))?;
    Ok((tmp, socket))
}

// The number on spacecrab's root line.
fn spacecrab_root(dir: &Path, args: &[&str]) -> io::Result<u64> {
    let output = spacecrab(dir).args(args).arg(".").output()?;
    assert_eq!(output.status.code(), Some(0), "{args:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let root = stdout.lines().last().unwrap();
    Ok(root.split_once('\t').unwrap().0.parse().unwrap())
}

// The number du prints for the whole folder.
fn du(dir: &Path, flag: &str) -> io::Result<u64> {
    let output = Command::new("du").arg(flag).arg(dir).output()?;
    assert!(output.status.success(), "du {flag}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout.split_whitespace().next().unwrap().parse().unwrap())
}

#[cfg(target_os = "linux")]
#[test]
fn root_matches_du() -> io::Result<()> {
    let (tmp, _socket) = fixture()?;
    let dir = tmp.path();

    assert_eq!(spacecrab_root(dir, &["-b"])?, du(dir, "-sB1")?);
    // Needs GNU du 9.2 or newer: older ones also count folders' lengths here.
    assert_eq!(spacecrab_root(dir, &["-A", "-b"])?, du(dir, "-sb")?);
    Ok(())
}

// BSD du -A still counts folders' own lengths, unlike GNU du, so only space on
// disk is compared here. -k rounds up per KiB, so we do too.
#[cfg(target_os = "macos")]
#[test]
fn root_matches_du_in_kib() -> io::Result<()> {
    let (tmp, _socket) = fixture()?;
    let dir = tmp.path();

    let root = spacecrab_root(dir, &["-b"])?;
    assert_eq!(root.div_ceil(1024), du(dir, "-sk")?);
    Ok(())
}
