use super::spacecrab;
use std::{fs, io, path::Path};
use tempfile::tempdir;

#[test]
fn bytes_lines_are_digits_then_tab() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("empty"), b"")?;
    fs::write(dir.join("a b.txt"), [0; 1000])?;
    fs::create_dir(dir.join("sub"))?;
    fs::write(dir.join("sub").join("c.txt"), vec![0; 1_048_565])?;

    for flag in ["-b", "--bytes"] {
        let output = spacecrab(dir).arg(flag).output()?;

        assert!(output.status.success(), "{flag}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.lines().count(), 4, "{flag}: {stdout}");
        // Same as ^[0-9]+\t, without pulling in a regex crate.
        for line in stdout.lines() {
            let (size, _) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("{flag}: {line:?}"));
            assert!(
                !size.is_empty() && size.bytes().all(|b| b.is_ascii_digit()),
                "{flag}: {line:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn bytes_are_exact_and_root_is_the_sum() -> io::Result<()> {
    let tmp = tempdir()?;
    let dir = tmp.path();

    fs::write(dir.join("a"), b"")?;
    fs::write(dir.join("b"), [0; 1000])?;
    fs::create_dir(dir.join("sub"))?;
    fs::write(dir.join("sub").join("c"), [0; 1536])?;
    fs::write(dir.join("sub").join("d"), vec![0; 1_048_565])?;

    let output = spacecrab(dir).arg("-b").output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut lines: Vec<&str> = stdout.lines().collect();
    let root = lines.pop().unwrap();
    lines.sort();

    let a = Path::new(".").join("a");
    let b = Path::new(".").join("b");
    let c = Path::new(".").join("sub").join("c");
    let d = Path::new(".").join("sub").join("d");
    // Sorted as text, so 1048565 comes before 1536.
    assert_eq!(
        lines,
        [
            format!("0\t{}", a.display()),
            format!("1000\t{}", b.display()),
            format!("1048565\t{}", d.display()),
            format!("1536\t{}", c.display()),
        ]
    );
    // 0 + 1000 + 1536 + 1048565. Folders don't add their own size yet.
    assert_eq!(root, "1051101\t.");
    Ok(())
}
