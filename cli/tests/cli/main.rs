mod args;
mod help;
mod report;
#[cfg(unix)]
mod scan_errors;
#[cfg(unix)]
mod streams;

use std::{path::Path, process::Command};

#[cfg(unix)]
use std::{fs, io, os::unix::fs::PermissionsExt, path::PathBuf};

fn spacecrab(dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_spacecrab"));
    command.current_dir(dir);
    command
}

// Sets a directory's mode and restores 0o755 on drop, so the temp dir
// can be deleted even when the test fails.
#[cfg(unix)]
struct Locked(PathBuf);

#[cfg(unix)]
impl Locked {
    fn new(dir: impl Into<PathBuf>, mode: u32) -> io::Result<Self> {
        let dir = dir.into();
        fs::set_permissions(&dir, fs::Permissions::from_mode(mode))?;
        Ok(Locked(dir))
    }
}

#[cfg(unix)]
impl Drop for Locked {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
    }
}
