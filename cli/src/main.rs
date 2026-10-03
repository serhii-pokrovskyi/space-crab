#![deny(clippy::print_stderr, clippy::print_stdout)]

mod formatter;

use clap::Parser;
use formatter::format_size;
use spacecrab_core::{EntryKind, scan};
use std::{
    collections::HashSet,
    fmt,
    io::{self, Write},
    path::{MAIN_SEPARATOR_STR, Path, PathBuf},
    process::ExitCode,
};

const AFTER_HELP: &str = "\
Sizes are space used on disk, as du counts it; -A counts file lengths instead.
A file with several hard links is counted once. On Windows sizes are always
file lengths, and each hard link counts separately.

Exit status:
  0  complete
  1  results incomplete, or the report couldn't be written
  2  usage error";

#[derive(Parser)]
#[command(version, about, after_help = AFTER_HELP)]
struct Args {
    /// File or directory to analyze
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Print sizes as exact byte counts
    #[arg(short, long)]
    bytes: bool,

    /// Count file lengths instead of space on disk
    #[arg(short = 'A', long)]
    apparent_size: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args.path, args.bytes, args.apparent_size) {
        Ok(code) => code,
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            print_error(format_args!("error writing output: {}", err));
            ExitCode::FAILURE
        }
    }
}

// A direct child of the root, with everything under it added up.
struct Child {
    size: u64,
    path: PathBuf,
    is_dir: bool,
}

fn run(root: &Path, bytes: bool, apparent: bool) -> io::Result<ExitCode> {
    // Totals of the entries on the current path, root first.
    let mut open: Vec<u64> = Vec::new();
    let mut children: Vec<Child> = Vec::new();
    let mut unread = Vec::new();
    let mut seen = HashSet::new();
    let mut total = 0;
    let mut failed = false;
    let mut root_failed = false;
    for entry in scan(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                root_failed |= err.path() == root;
                // A child we couldn't even stat still gets a line, with nothing
                // counted. An unreadable folder is already listed, though.
                let listed = children.last().is_some_and(|c| c.path == err.path());
                if err.path().parent() == Some(root) && !listed {
                    unread.push(err.path().to_path_buf());
                }
                print_error(err);
                failed = true;
                continue;
            }
        };
        close(&mut open, entry.depth(), &mut children, &mut total);
        // No disk size on Windows, so it's the file length there either way.
        // Only files and symlinks have a real length; a folder's st_size is
        // filesystem trivia, so like GNU du we count it as 0.
        let mut size = match entry.disk_size() {
            Some(size) if !apparent => size,
            _ if matches!(entry.kind(), EntryKind::File | EntryKind::Symlink) => {
                entry.apparent_size()
            }
            _ => 0,
        };
        // A file with several hard links only counts the first time we meet it.
        let linked = !entry.is_dir() && entry.link_count().is_some_and(|n| n > 1);
        if linked && entry.file_id().is_some_and(|id| !seen.insert(id)) {
            size = 0;
        }
        open.push(size);
        if entry.depth() == 1 {
            children.push(Child {
                size: 0,
                is_dir: entry.is_dir(),
                path: entry.into_path(),
            });
        }
    }
    close(&mut open, 0, &mut children, &mut total);
    children.extend(unread.into_iter().map(|path| Child {
        size: 0,
        path,
        is_dir: false,
    }));

    // Half a report for a root we couldn't read would just be wrong.
    if root_failed {
        return Ok(ExitCode::FAILURE);
    }
    // Smallest first, so the biggest ends up right above the total.
    children.sort_by(|a, b| {
        let (pa, pb) = (a.path.as_os_str(), b.path.as_os_str());
        a.size
            .cmp(&b.size)
            .then_with(|| pa.as_encoded_bytes().cmp(pb.as_encoded_bytes()))
    });
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    for child in &children {
        let sep = if child.is_dir { MAIN_SEPARATOR_STR } else { "" };
        let path = format!("{}{sep}", child.path.display());
        write_line(&mut stdout, child.size, path, bytes)?;
    }
    write_line(&mut stdout, total, root.display(), bytes)?;
    stdout.flush()?;
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

// The scan goes depth-first, so once it's back at `depth`, everything deeper is
// done. Each finished entry goes into its parent; a finished child of the root
// gets its final size.
fn close(open: &mut Vec<u64>, depth: usize, children: &mut [Child], total: &mut u64) {
    while open.len() > depth {
        let Some(size) = open.pop() else { return };
        match open.last_mut() {
            Some(parent) => *parent = parent.saturating_add(size),
            None => *total = size,
        }
        if open.len() == 1 {
            if let Some(child) = children.last_mut() {
                child.size = size;
            }
        }
    }
}

// Tab, not space: paths can have spaces too. Human sizes are padded to the
// width of "1023.99 KiB" so paths line up; -b stays bare for scripts.
fn write_line(
    out: &mut impl Write,
    size: u64,
    path: impl fmt::Display,
    bytes: bool,
) -> io::Result<()> {
    if bytes {
        writeln!(out, "{}\t{}", size, path)
    } else {
        writeln!(out, "{:>11}\t{}", format_size(size), path)
    }
}

fn print_error(msg: impl fmt::Display) {
    let _ = writeln!(io::stderr(), "spacecrab: {}", msg);
}
