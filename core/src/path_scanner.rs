use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn scan(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        // file_type() does not follow symlinks: a symlinked directory is listed, not entered.
        if entry.file_type()?.is_dir() {
            entries.extend(scan(&path)?);
        } else {
            entries.push(path);
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use std::{fs, io};
    use tempfile::tempdir;

    #[test]
    fn test_scan_files() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::File::create(dir.join("foo.txt"))?.write_all(b"hello")?;
        fs::create_dir_all(dir.join("sub/deeper"))?;
        fs::File::create(dir.join("sub/bar.log"))?.write_all(b"world")?;
        fs::File::create(dir.join("sub/deeper/baz.md"))?.write_all(b"!")?;

        let mut files = scan(dir)?;
        files.sort();

        let expected: Vec<PathBuf> = vec![
            dir.join("foo.txt"),
            dir.join("sub/bar.log"),
            dir.join("sub/deeper/baz.md"),
        ];

        assert_eq!(files, expected);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_does_not_follow_dir_symlinks() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::create_dir(dir.join("real"))?;
        fs::File::create(dir.join("real/foo.txt"))?.write_all(b"hello")?;
        std::os::unix::fs::symlink(dir.join("real"), dir.join("link"))?;

        let mut files = scan(dir)?;
        files.sort();

        assert_eq!(files, vec![dir.join("link"), dir.join("real/foo.txt")]);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_stops_at_symlink_loop() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        std::os::unix::fs::symlink(dir, dir.join("loop"))?;

        let files = scan(dir)?;

        assert_eq!(files, vec![dir.join("loop")]);
        Ok(())
    }
}
