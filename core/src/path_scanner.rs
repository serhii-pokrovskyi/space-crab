use crate::{Entry, Error};
use std::{
    fs, io,
    iter::FusedIterator,
    path::{Path, PathBuf},
    vec,
};

pub fn scan(root: impl AsRef<Path>) -> Scan {
    Scan {
        root: Some(root.as_ref().to_path_buf()),
        pending: None,
        levels: Vec::new(),
    }
}

#[derive(Debug)]
pub struct Scan {
    // The root, until the first call reads it.
    root: Option<PathBuf>,
    // The directory just yielded, with its depth: read on the next call.
    pending: Option<(PathBuf, usize)>,
    // Entries read but not yielded yet, one list per open directory, deepest last.
    levels: Vec<vec::IntoIter<Result<Entry, Error>>>,
}

impl Iterator for Scan {
    type Item = Result<Entry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.root.take() {
            // fs::metadata follows a root symlink, so `spacecrab link-to-dir` scans the directory.
            let root = match fs::metadata(&root) {
                Ok(metadata) => Ok(Entry::new(root, 0, &metadata)),
                Err(err) => Err(Error::new(root, err)),
            };
            self.levels.push(vec![root].into_iter());
        }
        if let Some((dir, depth)) = self.pending.take() {
            self.levels.push(read_children(&dir, depth + 1).into_iter());
        }
        while let Some(level) = self.levels.last_mut() {
            match level.next() {
                Some(Ok(entry)) => {
                    // Symlinks have their own kind, so a symlinked directory is never entered.
                    if entry.is_dir() {
                        self.pending = Some((entry.path().to_path_buf(), entry.depth()));
                    }
                    return Some(Ok(entry));
                }
                Some(Err(err)) => return Some(Err(err)),
                None => {
                    self.levels.pop();
                }
            }
        }
        None
    }
}

impl FusedIterator for Scan {}

fn read_children(dir: &Path, depth: usize) -> Vec<Result<Entry, Error>> {
    match fs::read_dir(dir) {
        Ok(entries) => collect_children(dir, depth, entries),
        Err(err) => vec![Err(Error::new(dir, err))],
    }
}

// Reads the whole listing so the directory is closed before any subdirectory is opened.
fn collect_children(
    dir: &Path,
    depth: usize,
    entries: impl Iterator<Item = io::Result<fs::DirEntry>>,
) -> Vec<Result<Entry, Error>> {
    let mut children = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                // DirEntry::metadata does not follow symlinks.
                children.push(match entry.metadata() {
                    Ok(metadata) => Ok(Entry::new(path, depth, &metadata)),
                    Err(err) => Err(Error::new(path, err)),
                });
            }
            Err(err) => {
                children.push(Err(Error::new(dir, err)));
                break;
            }
        }
    }
    children
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntryKind;
    use tempfile::tempdir;

    fn summary(entries: &[Entry], root: &Path) -> Vec<(PathBuf, usize, EntryKind)> {
        let mut summary: Vec<_> = entries
            .iter()
            .map(|e| {
                let path = e.path().strip_prefix(root).unwrap().to_path_buf();
                (path, e.depth(), e.kind())
            })
            .collect();
        summary.sort_by(|a, b| a.0.cmp(&b.0));
        summary
    }

    // Every directory's descendants come right after it, before anything else.
    fn assert_preorder(entries: &[Entry]) {
        for (i, dir) in entries.iter().enumerate().filter(|(_, e)| e.is_dir()) {
            let is_inside = |e: &Entry| e.path() != dir.path() && e.path().starts_with(dir.path());
            let run = entries[i + 1..].iter().take_while(|e| is_inside(e)).count();
            let all = entries.iter().filter(|e| is_inside(e)).count();
            assert_eq!(run, all, "{} is not contiguous", dir.path().display());
        }
    }

    #[test]
    fn test_scan_yields_root_then_contents_in_preorder() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::write(dir.join("foo.txt"), b"hello")?;
        fs::create_dir(dir.join("empty"))?;
        fs::create_dir_all(dir.join("sub/deeper"))?;
        fs::write(dir.join("sub/bar.log"), b"world")?;
        fs::write(dir.join("sub/deeper/baz.md"), b"!")?;

        let entries = scan(dir).collect::<Result<Vec<_>, _>>()?;

        assert_eq!(entries[0].path(), dir);
        assert_preorder(&entries);
        let p = PathBuf::from;
        assert_eq!(
            summary(&entries, dir),
            [
                (p(""), 0, EntryKind::Dir),
                (p("empty"), 1, EntryKind::Dir),
                (p("foo.txt"), 1, EntryKind::File),
                (p("sub"), 1, EntryKind::Dir),
                (p("sub/bar.log"), 2, EntryKind::File),
                (p("sub/deeper"), 2, EntryKind::Dir),
                (p("sub/deeper/baz.md"), 3, EntryKind::File),
            ]
        );
        let foo = entries.iter().find(|e| e.path() == dir.join("foo.txt"));
        assert_eq!(foo.unwrap().apparent_size(), 5);
        Ok(())
    }

    #[test]
    fn test_file_root_is_one_entry() -> io::Result<()> {
        let tmp = tempdir()?;
        let file = tmp.path().join("foo.txt");
        fs::write(&file, b"hello")?;

        let entries = scan(&file).collect::<Result<Vec<_>, _>>()?;

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path(), file);
        assert_eq!(entries[0].depth(), 0);
        assert_eq!(entries[0].kind(), EntryKind::File);
        assert_eq!(entries[0].apparent_size(), 5);
        Ok(())
    }

    #[test]
    fn test_missing_root_is_one_error() -> io::Result<()> {
        let tmp = tempdir()?;
        let missing = tmp.path().join("missing");

        let mut scan = scan(&missing);

        let err = scan.next().unwrap().unwrap_err();
        assert_eq!(err.path(), missing);
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(scan.next().is_none());
        assert!(scan.next().is_none());
        Ok(())
    }

    #[test]
    fn test_listing_error_stops_reading_dir() {
        let dir = Path::new("some/dir");
        let failing = std::iter::repeat_with(|| Err(io::Error::other("listing failed")));

        let children = collect_children(dir, 1, failing);

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].as_ref().unwrap_err().path(), dir);
    }

    #[cfg(unix)]
    #[test]
    fn test_root_symlink_is_followed() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::create_dir(dir.join("real"))?;
        fs::write(dir.join("real/foo.txt"), b"hello")?;
        std::os::unix::fs::symlink("real", dir.join("link"))?;

        let entries = scan(dir.join("link")).collect::<Result<Vec<_>, _>>()?;

        assert_eq!(
            summary(&entries, &dir.join("link")),
            [
                (PathBuf::from(""), 0, EntryKind::Dir),
                (PathBuf::from("foo.txt"), 1, EntryKind::File),
            ]
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_does_not_follow_dir_symlinks() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::create_dir(dir.join("real"))?;
        fs::write(dir.join("real/foo.txt"), b"hello")?;
        std::os::unix::fs::symlink(dir.join("real"), dir.join("link"))?;

        let entries = scan(dir).collect::<Result<Vec<_>, _>>()?;

        let p = PathBuf::from;
        assert_eq!(
            summary(&entries, dir),
            [
                (p(""), 0, EntryKind::Dir),
                (p("link"), 1, EntryKind::Symlink),
                (p("real"), 1, EntryKind::Dir),
                (p("real/foo.txt"), 2, EntryKind::File),
            ]
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_stops_at_symlink_loop() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        std::os::unix::fs::symlink(dir, dir.join("loop"))?;

        let entries = scan(dir).collect::<Result<Vec<_>, _>>()?;

        assert_eq!(
            summary(&entries, dir),
            [
                (PathBuf::from(""), 0, EntryKind::Dir),
                (PathBuf::from("loop"), 1, EntryKind::Symlink),
            ]
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_size_is_link_length() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::write(dir.join("target.bin"), [0; 1000])?;
        std::os::unix::fs::symlink("target.bin", dir.join("link"))?;
        std::os::unix::fs::symlink("missing", dir.join("dead"))?;

        let entries = scan(dir).collect::<Result<Vec<_>, _>>()?;

        // A symlink's own size is the length of the path it points to.
        let size = |name| {
            let entry = entries.iter().find(|e| e.path() == dir.join(name));
            entry.unwrap().apparent_size()
        };
        assert_eq!(size("link"), "target.bin".len() as u64);
        assert_eq!(size("dead"), "missing".len() as u64);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_fifo_and_socket_are_other_and_not_opened() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        let status = std::process::Command::new("mkfifo")
            .arg(dir.join("fifo"))
            .status()?;
        assert!(status.success());
        let _socket = std::os::unix::net::UnixListener::bind(dir.join("socket"))?;

        // Opening the FIFO would block, so finishing at all shows it was never opened.
        let entries = scan(dir).collect::<Result<Vec<_>, _>>()?;

        let p = PathBuf::from;
        assert_eq!(
            summary(&entries, dir),
            [
                (p(""), 0, EntryKind::Dir),
                (p("fifo"), 1, EntryKind::Other),
                (p("socket"), 1, EntryKind::Other),
            ]
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_continues_past_unreadable_dir() -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::write(dir.join("foo.txt"), b"hello")?;
        fs::create_dir(dir.join("locked"))?;
        fs::write(dir.join("locked/bar.txt"), b"world")?;
        fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000))?;

        let results: Vec<_> = scan(dir).collect();
        fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755))?;

        let locked = results
            .iter()
            .position(|r| r.as_ref().is_ok_and(|e| e.path() == dir.join("locked")))
            .unwrap();
        let err = results[locked + 1].as_ref().unwrap_err();
        assert_eq!(err.path(), dir.join("locked"));
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert!(err.io_error().raw_os_error().is_some());
        let prefix = format!("{}: ", dir.join("locked").display());
        assert!(err.to_string().starts_with(&prefix));

        assert_eq!(results.iter().filter(|r| r.is_err()).count(), 1);
        let ok: Vec<_> = results.into_iter().filter_map(Result::ok).collect();
        let p = PathBuf::from;
        assert_eq!(
            summary(&ok, dir),
            [
                (p(""), 0, EntryKind::Dir),
                (p("foo.txt"), 1, EntryKind::File),
                (p("locked"), 1, EntryKind::Dir),
            ]
        );
        Ok(())
    }
}
