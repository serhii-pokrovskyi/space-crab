use super::{size_of, spacecrab};
use std::{fs, io};
use tempfile::tempdir;

// The sizes from `-b` output: the lines above the root in printed order, and
// the root.
fn sizes(stdout: &[u8]) -> (Vec<u64>, u64) {
    let stdout = String::from_utf8(stdout.to_vec()).unwrap();
    let mut sizes: Vec<u64> = stdout
        .lines()
        .map(|line| line.split_once('\t').unwrap().0.parse().unwrap())
        .collect();
    let root = sizes.pop().unwrap();
    (sizes, root)
}

#[cfg(unix)]
#[test]
fn hard_links_count_once() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();
    fs::write(dir.join("big"), vec![1; 10 << 20])?;
    fs::hard_link(dir.join("big"), dir.join("link1"))?;
    fs::hard_link(dir.join("big"), dir.join("link2"))?;

    // Whichever link the scan meets first carries the bytes; the rest count 0.
    let output = spacecrab(dir).arg("-b").output()?;
    assert!(output.status.success());
    let big = size_of(dir.join("big"));
    assert_eq!(sizes(&output.stdout), (vec![0, 0, big], size_of(dir) + big));

    let output = spacecrab(dir).args(["-A", "-b"]).output()?;
    assert!(output.status.success());
    let dir_len = fs::symlink_metadata(dir)?.len();
    let expected = (vec![0, 0, 10 << 20], dir_len + (10 << 20));
    assert_eq!(sizes(&output.stdout), expected);
    Ok(())
}

#[cfg(unix)]
#[test]
fn hard_link_in_sibling_folder_counts_in_only_one() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();
    fs::create_dir(dir.join("a"))?;
    fs::create_dir(dir.join("b"))?;
    fs::write(dir.join("a/f"), vec![1; 1 << 20])?;
    fs::hard_link(dir.join("a/f"), dir.join("b/f"))?;

    let output = spacecrab(dir).arg("-b").output()?;

    assert!(output.status.success());
    let s = |name: &str| size_of(dir.join(name));
    let (a, b, f) = (s("a"), s("b"), s("a/f"));
    let (lines, root) = sizes(&output.stdout);
    let sorted = |mut v: Vec<u64>| {
        v.sort();
        v
    };
    assert!(
        lines == sorted(vec![a + f, b]) || lines == sorted(vec![a, b + f]),
        "{lines:?}"
    );
    assert_eq!(root, size_of(dir) + a + b + f);
    Ok(())
}

#[cfg(windows)]
#[test]
fn hard_links_count_each_on_windows() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();
    fs::write(dir.join("a"), [1; 1000])?;
    fs::hard_link(dir.join("a"), dir.join("b"))?;

    let output = spacecrab(dir).arg("-b").output()?;

    assert!(output.status.success());
    let expected = (vec![1000, 1000], size_of(dir) + 2000);
    assert_eq!(sizes(&output.stdout), expected);
    Ok(())
}
