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
}

// st_blocks is in 512-byte units, whatever the filesystem's block size is.
#[cfg(unix)]
fn disk_size(metadata: &fs::Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(metadata.blocks().saturating_mul(512))
}

// Windows would need FFI for this, and core stays std-only.
#[cfg(not(unix))]
fn disk_size(_: &fs::Metadata) -> Option<u64> {
    None
}
