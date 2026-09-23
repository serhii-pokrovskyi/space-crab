use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn scan(dir: &Path) -> impl Iterator<Item = io::Result<PathBuf>> + use<> {
    Scan {
        dirs: vec![dir.to_path_buf()],
        current: None,
    }
}

struct Scan {
    // Directories found but not read yet.
    dirs: Vec<PathBuf>,
    // The directory being read.
    current: Option<(PathBuf, fs::ReadDir)>,
}

impl Iterator for Scan {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((dir, entries)) = &mut self.current else {
                let dir = self.dirs.pop()?;
                match fs::read_dir(&dir) {
                    Ok(entries) => self.current = Some((dir, entries)),
                    Err(err) => return Some(Err(with_path(&dir, err))),
                }
                continue;
            };
            let entry = match entries.next() {
                Some(Ok(entry)) => entry,
                Some(Err(err)) => return Some(Err(with_path(dir, err))),
                None => {
                    self.current = None;
                    continue;
                }
            };
            let path = entry.path();
            match entry.file_type() {
                // file_type() does not follow symlinks: a symlinked directory is listed, not entered.
                Ok(file_type) if file_type.is_dir() => self.dirs.push(path),
                Ok(_) => return Some(Ok(path)),
                Err(err) => return Some(Err(with_path(&path, err))),
            }
        }
    }
}

// io::Error does not say which path failed, so put it in the message.
fn with_path(path: &Path, err: io::Error) -> io::Error {
    io::Error::new(err.kind(), format!("{}: {}", path.display(), err))
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

        let mut files = scan(dir).collect::<io::Result<Vec<_>>>()?;
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

        let mut files = scan(dir).collect::<io::Result<Vec<_>>>()?;
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

        let files = scan(dir).collect::<io::Result<Vec<_>>>()?;

        assert_eq!(files, vec![dir.join("loop")]);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_continues_past_unreadable_dir() -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::File::create(dir.join("foo.txt"))?.write_all(b"hello")?;
        fs::create_dir(dir.join("locked"))?;
        fs::File::create(dir.join("locked/bar.txt"))?.write_all(b"world")?;
        fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000))?;

        let mut files = Vec::new();
        let mut errors = Vec::new();
        for result in scan(dir) {
            match result {
                Ok(path) => files.push(path),
                Err(err) => errors.push(err),
            }
        }
        fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755))?;

        assert_eq!(files, vec![dir.join("foo.txt")]);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), io::ErrorKind::PermissionDenied);
        let prefix = format!("{}: ", dir.join("locked").display());
        assert!(errors[0].to_string().starts_with(&prefix));
        Ok(())
    }
}
