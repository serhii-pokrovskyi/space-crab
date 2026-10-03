use super::{size_of, spacecrab};
use std::{
    fs, io,
    path::{MAIN_SEPARATOR, Path},
};
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

    let s = |name: &str| size_of(dir.join(name));
    let a = Path::new(".").join("a");
    let b = Path::new(".").join("b");
    let sub = Path::new(".").join("sub");
    let sub_size = s("sub") + s("sub/c") + s("sub/d");
    // The root is its own size plus every line above it.
    let root_size = size_of(dir) + s("a") + s("b") + sub_size;
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "{}\t{}\n{}\t{}\n{sub_size}\t{}{MAIN_SEPARATOR}\n{root_size}\t.\n",
            s("a"),
            a.display(),
            s("b"),
            b.display(),
            sub.display()
        )
    );
    Ok(())
}
