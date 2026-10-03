use std::{
    fs,
    path::{Path, PathBuf},
};

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

// Device and inode, kept private so callers can only compare ids.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileId {
    dev: u64,
    ino: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EntryKind {
    File,
    Dir,
    Symlink,
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path(self) -> PathBuf {
        self.path
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn kind(&self) -> EntryKind {
        self.kind
    }

    pub fn is_dir(&self) -> bool {
        self.kind == EntryKind::Dir
    }

    pub fn apparent_size(&self) -> u64 {
        self.apparent_size
    }

    pub fn disk_size(&self) -> Option<u64> {
        self.disk_size
    }

    pub fn file_id(&self) -> Option<FileId> {
        self.file_id
    }

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
