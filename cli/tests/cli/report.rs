use super::{size_of, spacecrab};
use std::{
    fs, io,
    path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, Path, PathBuf},
};
use tempfile::tempdir;

// The `-b` report we expect: children smallest first, ties by path bytes, then
// the root with its own size plus all of theirs.
fn expected_report(root: &str, root_size: u64, mut children: Vec<(u64, PathBuf, bool)>) -> String {
    children.sort_by(|a, b| {
        let (pa, pb) = (a.1.as_os_str(), b.1.as_os_str());
        a.0.cmp(&b.0)
            .then_with(|| pa.as_encoded_bytes().cmp(pb.as_encoded_bytes()))
    });
    let mut report = String::new();
    let mut total = root_size;
    for (size, path, is_dir) in &children {
        let sep = if *is_dir { MAIN_SEPARATOR_STR } else { "" };
        report += &format!("{size}\t{}{sep}\n", path.display());
        total += size;
    }
    report + &format!("{total}\t{root}\n")
}

#[test]
fn children_with_folder_totals_sorted() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::write(dir.join(".hidden"), [0; 10])?;
    fs::create_dir(dir.join("empty"))?;
    fs::create_dir_all(dir.join("sub/deeper"))?;
    fs::write(dir.join("sub/y"), [0; 200])?;
    fs::write(dir.join("sub/deeper/x"), [0; 500])?;
    // Same size, so the path has to decide. Byte order is B, D, a, c, which
    // isn't the order they're made in, its reverse, or Windows' name order, so
    // a listing order can't pass for the tie-break.
    fs::write(dir.join("tie_a"), [0; 300])?;
    fs::write(dir.join("tie_B"), [0; 300])?;
    fs::write(dir.join("tie_c"), [0; 300])?;
    fs::write(dir.join("tie_D"), [0; 300])?;

    let s = |name: &str| size_of(dir.join(name));
    let p = |name: &str| Path::new(".").join(name);
    let sub = s("sub") + s("sub/y") + s("sub/deeper") + s("sub/deeper/x");
    let expected = expected_report(
        ".",
        size_of(dir),
        vec![
            (s("a.txt"), p("a.txt"), false),
            (s(".hidden"), p(".hidden"), false),
            (s("empty"), p("empty"), true),
            (sub, p("sub"), true),
            (s("tie_a"), p("tie_a"), false),
            (s("tie_B"), p("tie_B"), false),
            (s("tie_c"), p("tie_c"), false),
            (s("tie_D"), p("tie_D"), false),
        ],
    );
    for _ in 0..3 {
        let output = spacecrab(dir).arg("-b").output()?;

        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_child_is_listed_not_followed() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::create_dir(dir.join("real"))?;
    fs::write(dir.join("real/a.txt"), [0; 1000])?;
    std::os::unix::fs::symlink("real", dir.join("link"))?;

    let output = spacecrab(dir).arg("-b").output()?;

    assert!(output.status.success());
    let s = |name: &str| size_of(dir.join(name));
    let p = |name: &str| Path::new(".").join(name);
    let expected = expected_report(
        ".",
        size_of(dir),
        vec![
            (s("link"), p("link"), false),
            (s("real") + s("real/a.txt"), p("real"), true),
        ],
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    Ok(())
}

#[test]
fn lines_split_at_first_tab_with_root_last() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a b.txt"), [0; 10])?;
    fs::create_dir(dir.join("my sub"))?;
    fs::write(dir.join("my sub").join("c d.txt"), [0; 20])?;

    let output = spacecrab(dir).output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    // A blank line or a header has no tab, so it would panic here.
    let lines: Vec<(&str, &str)> = stdout
        .lines()
        .map(|line| line.split_once('\t').unwrap_or_else(|| panic!("{line:?}")))
        .collect();
    let a = Path::new(".").join("a b.txt").display().to_string();
    let sub = Path::new(".").join("my sub");
    let sub = format!("{}{MAIN_SEPARATOR}", sub.display());
    let paths: Vec<&str> = lines.iter().map(|&(_, path)| path).collect();
    assert_eq!(paths, [a.as_str(), sub.as_str(), "."]);
    assert_eq!(lines[0].0, "       10 B");
    Ok(())
}

#[test]
fn human_sizes_are_right_aligned() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a"), b"")?;
    fs::write(dir.join("b"), [0; 1000])?;
    fs::write(dir.join("c"), [0; 1536])?;
    fs::write(dir.join("d"), vec![0; 1_048_565])?;

    let output = spacecrab(dir).output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut lines: Vec<&str> = stdout.lines().collect();
    let root = lines.pop().unwrap();
    let path = |name| Path::new(".").join(name).display().to_string();
    assert_eq!(
        lines,
        [
            format!("        0 B\t{}", path("a")),
            format!("     1000 B\t{}", path("b")),
            format!("   1.50 KiB\t{}", path("c")),
            format!("1023.99 KiB\t{}", path("d")),
        ]
    );
    // The root adds the folder's own size, which differs between filesystems.
    let (size, path) = root.split_once('\t').unwrap();
    assert_eq!((size.len(), path), (11, "."));
    Ok(())
}

#[test]
fn empty_dir_prints_only_the_root_line() -> io::Result<()> {
    let tmp = tempdir()?;

    let output = spacecrab(tmp.path()).arg("-b").output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, format!("{}\t.\n", size_of(tmp.path())));
    Ok(())
}

#[test]
fn root_line_is_path_as_typed() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::create_dir(dir.join("sub"))?;
    fs::write(dir.join("sub").join("a.txt"), [0; 1000])?;

    let sub = size_of(dir.join("sub"));
    let a = size_of(dir.join("sub").join("a.txt"));
    let dot = vec![(sub + a, Path::new(".").join("sub"), true)];
    let typed = vec![(a, Path::new("sub/").join("a.txt"), false)];
    let cases = [
        (&["-b"][..], expected_report(".", size_of(dir), dot)),
        (&["-b", "sub/"][..], expected_report("sub/", sub, typed)),
    ];
    for (args, expected) in cases {
        let output = spacecrab(dir).args(args).output()?;

        assert!(output.status.success(), "{args:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout, expected, "{args:?}");
    }
    Ok(())
}
