use std::{fs, io, path::Path};

pub fn file_size(path: impl AsRef<Path>) -> io::Result<u64> {
    let size = fs::symlink_metadata(path)?.len();
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[cfg(unix)]
    #[test]
    fn test_size_of_file_symlink_is_link_size() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::File::create(dir.join("target.bin"))?.write_all(&[0; 1000])?;
        std::os::unix::fs::symlink("target.bin", dir.join("link"))?;

        // A symlink's own size is the length of the path it points to.
        let link_size = "target.bin".len() as u64;
        assert_eq!(file_size(dir.join("link"))?, link_size);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_size_of_dangling_symlink() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        std::os::unix::fs::symlink("missing", dir.join("dead"))?;

        let link_size = "missing".len() as u64;
        assert_eq!(file_size(dir.join("dead"))?, link_size);
        Ok(())
    }
}
