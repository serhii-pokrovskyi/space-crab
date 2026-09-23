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
