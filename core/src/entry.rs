use std::{
    fs,
    path::{Path, PathBuf},
};

/// One thing a scan found: the root, or something under it.
#[derive(Debug)]
pub struct Entry {
    path: PathBuf,
    depth: usize,
    kind: EntryKind,
    apparent_size: u64,
    disk_size: Option<u64>,
    file_id: Option<FileId>,
    link_count: Option<u64>,
}

/// Which file an entry is, as of the scan. Hard links to the same file have
/// equal ids. It can only be compared and hashed.
// Device and inode, kept private so callers can only compare ids.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileId {
    dev: u64,
    ino: u64,
}

/// What an entry is. More kinds may come later, so match with a `_` arm.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A folder. The scan goes into it right after yielding it.
    Dir,
    /// A symbolic link, never followed. On Windows that includes junctions
    /// and folders a volume is mounted on. A root symlink is followed, so the
    /// root is never one.
    Symlink,
    /// Anything else: FIFOs, sockets, device files. Never opened.
    Other,
}

impl Entry {
    pub(crate) fn new(path: PathBuf, depth: usize, metadata: &fs::Metadata) -> Self {
        let file_type = metadata.file_type();
        let kind = if file_type.is_dir() {
            EntryKind::Dir
        } else if file_type.is_file() {
            EntryKind::File
        } else if file_type.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::Other
        };
        Entry {
            path,
            depth,
            kind,
            apparent_size: metadata.len(),
            disk_size: disk_size(metadata),
            file_id: file_id(metadata),
            link_count: link_count(metadata),
        }
    }

    /// The path: the root as it was given, joined with each name on the way
    /// down. It's never made absolute or resolved.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The path, without copying it.
    pub fn into_path(self) -> PathBuf {
        self.path
    }

    /// How far below the root this is: 0 for the root, 1 for what's directly
    /// in it, and so on.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// What this is: a file, a folder, a symlink or something else.
    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Whether this is a folder. A symlink to a folder isn't, except as the
    /// root, which is followed.
    pub fn is_dir(&self) -> bool {
        self.kind == EntryKind::Dir
    }

    /// The size the filesystem reports (`st_size`), as is:
    /// - for a file, its length in bytes;
    /// - for a symlink, the length of the path it points to (0 on Windows; for
    ///   a root symlink, which is followed, the target's size);
    /// - for a folder, a filesystem-specific number that says little;
    /// - for anything else, usually 0.
    ///
    /// Each hard link reports the whole file.
    pub fn apparent_size(&self) -> u64 {
        self.apparent_size
    }

    /// The space this takes on disk, as `st_blocks * 512`: a sparse file only
    /// counts what's allocated, a folder counts its own blocks (0 on APFS),
    /// not what's in it, and a symlink counts its own blocks (often 0), not
    /// its target's. Each hard link reports the whole file; use
    /// [`file_id`](Self::file_id) to count it once.
    ///
    /// `None` on Windows and other non-Unix targets, where std can't tell.
    pub fn disk_size(&self) -> Option<u64> {
        self.disk_size
    }

    /// Which file this is. Hard links to the same file have equal ids, so a
    /// set of the ids seen so far counts each file once.
    ///
    /// `None` on Windows and other non-Unix targets.
    pub fn file_id(&self) -> Option<FileId> {
        self.file_id
    }

    /// How many hard links the file has (`st_nlink`). A file with more than
    /// one can turn up more than once in a scan, once per link. So can
    /// anything under a filesystem mounted twice, like the data volume when
    /// scanning `/` on a Mac. Folders report a filesystem-specific number.
    ///
    /// `None` on Windows and other non-Unix targets.
    pub fn link_count(&self) -> Option<u64> {
        self.link_count
    }
}

// st_blocks is in 512-byte units, whatever the filesystem's block size is.
#[cfg(unix)]
fn disk_size(metadata: &fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.blocks().saturating_mul(512))
}

#[cfg(unix)]
fn file_id(metadata: &fs::Metadata) -> Option<FileId> {
    use std::os::unix::fs::MetadataExt;
    Some(FileId {
        dev: metadata.dev(),
        ino: metadata.ino(),
    })
}

#[cfg(unix)]
fn link_count(metadata: &fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.nlink())
}

// Windows would need FFI for these, and core stays std-only.
#[cfg(not(unix))]
fn disk_size(_: &fs::Metadata) -> Option<u64> {
    None
}

#[cfg(not(unix))]
fn file_id(_: &fs::Metadata) -> Option<FileId> {
    None
}

#[cfg(not(unix))]
fn link_count(_: &fs::Metadata) -> Option<u64> {
    None
}
