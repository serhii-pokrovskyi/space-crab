use super::spacecrab;
use std::{fs, io, path::Path};
use tempfile::tempdir;

// Root line is always last. The rest come in scan order, so sort them by path.
fn split_report(stdout: &str) -> (Vec<&str>, &str) {
    let mut lines: Vec<&str> = stdout.lines().collect();
    let root = lines.pop().unwrap();
    lines.sort_by_key(|&line| line.split_once('\t').map(|(_, path)| path));
    (lines, root)
}

#[test]
fn total_is_sum_of_listed_sizes() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a.txt"), [0; 1000])?;
    fs::create_dir(dir.join("sub"))?;
    fs::write(dir.join("sub/b.txt"), [0; 536])?;

    let output = spacecrab(dir).output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let (lines, root) = split_report(&stdout);
    let a = Path::new(".").join("a.txt");
    let b = Path::new(".").join("sub").join("b.txt");
    assert_eq!(
        lines,
        [
            format!("     1000 B\t{}", a.display()),
            format!("      536 B\t{}", b.display())
        ]
    );
    assert_eq!(root, "   1.50 KiB\t.");
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
    let (lines, root) = split_report(&stdout);
    // A blank line or a header has no tab, so it would panic here.
    let split = |line: &str| -> (String, String) {
        let (size, path) = line.split_once('\t').unwrap_or_else(|| panic!("{line:?}"));
        (size.to_string(), path.to_string())
    };
    let a = Path::new(".").join("a b.txt").display().to_string();
    let c = Path::new(".").join("my sub").join("c d.txt");
    assert_eq!(
        lines.into_iter().map(split).collect::<Vec<_>>(),
        [
            ("       10 B".to_string(), a),
            ("       20 B".to_string(), c.display().to_string())
        ]
    );
    assert_eq!(split(root), ("       30 B".to_string(), ".".to_string()));
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
    let (lines, root) = split_report(&stdout);
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
    // 1 051 101 bytes.
    assert_eq!(root, "   1.00 MiB\t.");
    Ok(())
}

#[test]
fn empty_dir_prints_only_the_root_line() -> io::Result<()> {
    let tmp = tempdir()?;

    let output = spacecrab(tmp.path()).output()?;

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "        0 B\t.\n");
    Ok(())
}

#[test]
fn root_line_is_path_as_typed() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::create_dir(dir.join("sub"))?;
    fs::write(dir.join("sub").join("a.txt"), [0; 1000])?;

    let cases = [
        (&[][..], ".", Path::new(".").join("sub").join("a.txt")),
        (&["sub/"][..], "sub/", Path::new("sub/").join("a.txt")),
    ];
    for (args, root, a) in cases {
        let output = spacecrab(dir).args(args).output()?;

        assert!(output.status.success(), "{args:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout,
            format!("     1000 B\t{}\n     1000 B\t{root}\n", a.display()),
            "{args:?}"
        );
    }
    Ok(())
}
