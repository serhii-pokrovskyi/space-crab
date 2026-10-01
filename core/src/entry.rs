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
}
