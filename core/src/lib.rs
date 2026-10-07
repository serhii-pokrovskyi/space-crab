//! Walks a folder and measures what's in it. This is the scanning part of the
//! [spacecrab] disk usage tool. It only uses std and runs on one thread.
//!
//! [`scan`] yields an [`Entry`] for the root and for everything under it, each
//! folder right before its contents. An entry has its path, depth and kind,
//! its file length ([`Entry::apparent_size`]) and, on Unix, the space it takes
//! on disk ([`Entry::disk_size`]) and its identity, for counting hard links
//! once ([`Entry::file_id`]). Anything that can't be read comes back as an
//! [`Error`] with its path, and the scan carries on.
//!
//! ```
//! use spacecrab_core::scan;
//!
//! # fn main() -> std::io::Result<()> {
//! let dir = tempfile::tempdir()?;
//! std::fs::write(dir.path().join("notes.txt"), "hello")?;
//!
//! let mut total = 0;
//! for entry in scan(dir.path()) {
//!     match entry {
//!         // There's no disk size on Windows, so fall back to the length.
//!         // Hard links count once per link here; see Entry::file_id.
//!         Ok(entry) => total += entry.disk_size().unwrap_or(entry.apparent_size()),
//!         Err(err) => eprintln!("{err}"),
//!     }
//! }
//! assert!(total > 0);
//! # Ok(())
//! # }
//! ```
//!
//! # Stability
//!
//! SemVer doesn't cover two things, which may change in any release: the text
//! of [`Error`]'s message, and the order of siblings in a scan.
//!
//! [spacecrab]: https://crates.io/crates/spacecrab

mod entry;
mod error;
mod path_scanner;

pub use entry::{Entry, EntryKind, FileId};
pub use error::Error;
pub use path_scanner::{Scan, ScanOptions, scan, scan_with};

// Fails to compile if any of these stops being Send + Sync, which would break
// anyone who moves them between threads.
const _: () = {
    const fn send_sync<T: Send + Sync>() {}
    send_sync::<Scan>();
    send_sync::<Entry>();
    send_sync::<Error>();
    send_sync::<FileId>();
    send_sync::<ScanOptions>();
};
