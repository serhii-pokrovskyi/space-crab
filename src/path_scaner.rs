use std::{
    fs, io, path::{Path, PathBuf}
};

pub trait Scanner {
    fn scan(&self, dir: &Path) -> io::Result<Vec<PathBuf>>;
}

pub struct FilesScaner;

impl Scanner for FilesScaner {
    fn scan(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                entries.extend(self.scan(&path)?);
            } else {
                entries.push(path);
            }
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    #[test]
    fn test_scan_files() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::File::create(dir.join("foo.txt"))?
            .write_all(b"hello")?;
        fs::create_dir_all(dir.join("sub/deeper"))?;
        fs::File::create(dir.join("sub/bar.log"))?
            .write_all(b"world")?;
        fs::File::create(dir.join("sub/deeper/baz.md"))?
            .write_all(b"!")?;

        let scaner = FilesScaner;
        let mut files = scaner.scan(dir)?;
        files.sort();

        let expected: Vec<PathBuf> = vec![
            dir.join("foo.txt"),
            dir.join("sub/bar.log"),
            dir.join("sub/deeper/baz.md"),
        ];

        assert_eq!(files, expected);
        Ok(())
    }
}