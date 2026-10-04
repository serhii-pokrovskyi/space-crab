use crate::{Entry, Error};
use std::{
    fs, io,
    iter::FusedIterator,
    path::{Path, PathBuf},
    vec,
};

/// Settings for [`scan_with`]. The default is what [`scan`] uses: go into
/// other filesystems too, on Unix.
#[derive(Clone, Debug, Default)]
pub struct ScanOptions {
    same_file_system: bool,
}

impl ScanOptions {
    /// Stay on the root's filesystem, like `du -x`. Anything on another
    /// device than the root, folder or file, is left out: not yielded, not
    /// gone into. The root's device is read after following a root symlink.
    ///
    /// It compares device ids, and on a Mac the system and data volumes share
    /// one, so `/` still goes into /System/Volumes/Data.
    ///
    /// Does nothing on Windows and other non-Unix targets.
    pub fn same_file_system(mut self, yes: bool) -> Self {
        self.same_file_system = yes;
        self
    }
}

/// Walks `root` and everything under it, going into other filesystems too, on
/// Unix. Same as [`scan_with`] with [`ScanOptions::default()`].
pub fn scan(root: impl AsRef<Path>) -> Scan {
    scan_with(root, ScanOptions::default())
}

/// Walks `root` and everything under it.
///
/// The root comes first, at depth 0. After that it's depth-first: each folder
/// comes right before its contents. Siblings come in whatever order the OS
/// lists them, which isn't specified.
///
/// A symlink as the root is followed, so a link to a folder scans the folder.
/// Symlinks below the root are yielded but never followed. A file as the root
/// yields just that file.
///
/// Errors don't stop the scan:
/// - if the root can't be read at all, there's one error and nothing else;
/// - a folder that can't be listed is yielded, then an error with its path;
/// - an entry whose metadata can't be read is an error with its path, instead
///   of an [`Entry`];
/// - if listing a folder fails part-way, that's an error with the folder's
///   path, and the rest of that folder is skipped.
///
/// Each folder's listing is read in full before going into a subfolder, so
/// memory grows with the size of the folders on the current path.
pub fn scan_with(root: impl AsRef<Path>, options: ScanOptions) -> Scan {
    Scan {
        root: Some(root.as_ref().to_path_buf()),
        options,
        stay_on: None,
        pending: None,
        levels: Vec::new(),
    }
}

/// The iterator [`scan`] and [`scan_with`] return. Once it's done, it keeps
/// returning `None`.
#[derive(Debug)]
pub struct Scan {
    // The root, until the first call reads it.
    root: Option<PathBuf>,
    options: ScanOptions,
    // The root's device when staying on one filesystem, once the root is read.
    stay_on: Option<u64>,
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
                Ok(metadata) => {
                    if self.options.same_file_system {
                        self.stay_on = device(&metadata);
                    }
                    Ok(Entry::new(root, 0, &metadata))
                }
                Err(err) => Err(Error::new(root, err)),
            };
            self.levels.push(vec![root].into_iter());
        }
        if let Some((dir, depth)) = self.pending.take() {
            let children = read_children(&dir, depth + 1, self.stay_on);
            self.levels.push(children.into_iter());
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

fn read_children(dir: &Path, depth: usize, stay_on: Option<u64>) -> Vec<Result<Entry, Error>> {
    match fs::read_dir(dir) {
        Ok(entries) => collect_children(dir, depth, stay_on, entries),
        Err(err) => vec![Err(Error::new(dir, err))],
    }
}

// Reads the whole listing so the directory is closed before any subdirectory is opened.
fn collect_children(
    dir: &Path,
    depth: usize,
    stay_on: Option<u64>,
    entries: impl Iterator<Item = io::Result<fs::DirEntry>>,
) -> Vec<Result<Entry, Error>> {
    let mut children = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                // DirEntry::metadata does not follow symlinks.
                match entry.metadata() {
                    // Anything on another filesystem is left out entirely, like du -x.
                    Ok(metadata) if on_other_device(stay_on, device(&metadata)) => {}
                    Ok(metadata) => children.push(Ok(Entry::new(path, depth, &metadata))),
                    Err(err) => children.push(Err(Error::new(path, err))),
                }
            }
            Err(err) => {
                children.push(Err(Error::new(dir, err)));
                break;
            }
        }
    }
    children
}

// Both devices have to be known. On Windows they never are, so nothing is skipped.
fn on_other_device(stay_on: Option<u64>, dev: Option<u64>) -> bool {
    matches!((stay_on, dev), (Some(root), Some(dev)) if root != dev)
}

#[cfg(unix)]
fn device(metadata: &fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.dev())
}

#[cfg(not(unix))]
fn device(_: &fs::Metadata) -> Option<u64> {
    None
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

        let children = collect_children(dir, 1, None, failing);

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].as_ref().unwrap_err().path(), dir);
    }

    #[test]
    fn test_other_device_needs_both_ids_and_a_difference() {
        assert!(!on_other_device(Some(1), Some(1)));
        assert!(on_other_device(Some(1), Some(2)));
        // Option off, or no device ids at all as on Windows: never skip.
        assert!(!on_other_device(None, Some(2)));
        assert!(!on_other_device(Some(1), None));
        assert!(!on_other_device(None, None));
    }

    #[test]
    fn test_same_file_system_keeps_a_single_filesystem_whole() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::write(dir.join("foo.txt"), b"hello")?;
        fs::create_dir_all(dir.join("sub/deeper"))?;
        fs::write(dir.join("sub/deeper/bar.txt"), b"world")?;

        let all = scan(dir).collect::<Result<Vec<_>, _>>()?;
        let options = ScanOptions::default().same_file_system(true);
        let same = scan_with(dir, options).collect::<Result<Vec<_>, _>>()?;

        assert_eq!(summary(&same, dir), summary(&all, dir));
        assert_eq!(same.len(), 5);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_same_file_system_leaves_out_other_devices() -> io::Result<()> {
        use std::os::unix::fs::MetadataExt;

        let tmp = tempdir()?;
        let dir = tmp.path();
        fs::write(dir.join("foo.txt"), b"hello")?;
        fs::create_dir(dir.join("sub"))?;
        let dev = fs::metadata(dir)?.dev();

        // Pretend the root is on some other device than everything in here.
        assert_eq!(read_children(dir, 1, Some(dev)).len(), 2);
        assert!(read_children(dir, 1, Some(dev.wrapping_add(1))).is_empty());

        // The root's device is only kept when the option is on.
        let mut on = scan_with(dir, ScanOptions::default().same_file_system(true));
        on.next();
        assert_eq!(on.stay_on, Some(dev));
        let mut off = scan(dir);
        off.next();
        assert_eq!(off.stay_on, None);
        Ok(())
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
    fn test_disk_size_counts_allocated_blocks() -> io::Result<()> {
        use std::os::unix::fs::MetadataExt;

        let tmp = tempdir()?;
        let sparse = tmp.path().join("sparse");
        fs::File::create(&sparse)?.set_len(1 << 30)?;
        let written = tmp.path().join("written");
        fs::write(&written, [1; 1000])?;

        let sparse_entry = scan(&sparse).next().unwrap()?;
        let written_entry = scan(&written).next().unwrap()?;

        // Nothing was written, so next to nothing is allocated.
        assert!(sparse_entry.disk_size().unwrap() < 1 << 20);
        assert_eq!(sparse_entry.apparent_size(), 1 << 30);
        // st_blocks counts 512-byte units, whatever the block size.
        let blocks = fs::symlink_metadata(&written)?.blocks();
        assert!(blocks > 0);
        assert_eq!(written_entry.disk_size(), Some(blocks * 512));
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn test_disk_size_is_none_on_windows() -> io::Result<()> {
        let tmp = tempdir()?;
        let file = tmp.path().join("foo.txt");
        fs::write(&file, b"hello")?;

        let entry = scan(&file).next().unwrap()?;

        assert_eq!(entry.disk_size(), None);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_hard_links_share_file_id() -> io::Result<()> {
        let tmp = tempdir()?;
        let dir = tmp.path();

        fs::write(dir.join("a"), b"hello")?;
        fs::hard_link(dir.join("a"), dir.join("b"))?;
        fs::write(dir.join("c"), b"hello")?;

        let entries = scan(dir).collect::<Result<Vec<_>, _>>()?;

        let entry = |name| entries.iter().find(|e| e.path() == dir.join(name)).unwrap();
        assert_eq!(entry("a").file_id(), entry("b").file_id());
        assert_ne!(entry("a").file_id(), entry("c").file_id());
        assert_eq!(entry("a").link_count(), Some(2));
        assert_eq!(entry("c").link_count(), Some(1));
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn test_file_id_and_link_count_are_none_on_windows() -> io::Result<()> {
        let tmp = tempdir()?;
        let file = tmp.path().join("foo.txt");
        fs::write(&file, b"hello")?;

        let entry = scan(&file).next().unwrap()?;

        assert_eq!(entry.file_id(), None);
        assert_eq!(entry.link_count(), None);
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
