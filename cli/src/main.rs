#![deny(clippy::print_stderr, clippy::print_stdout)]

mod formatter;

use clap::Parser;
use formatter::format_size;
use spacecrab_core::scan;
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const AFTER_HELP: &str = "\
Sizes are file lengths (apparent size), not the space used on disk.

Exit status:
  0  complete
  1  results incomplete, or the report couldn't be written
  2  usage error";

#[derive(Parser)]
#[command(version, about, after_help = AFTER_HELP)]
struct Args {
    /// Directory to analyze
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args.path) {
        Ok(code) => code,
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            print_error(format_args!("error writing output: {}", err));
            ExitCode::FAILURE
        }
    }
}

fn run(root: &Path) -> io::Result<ExitCode> {
    if let Err(err) = fs::read_dir(root) {
        print_error(format_args!("{}: {}", root.display(), err));
        return Ok(ExitCode::FAILURE);
    }
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut total = 0;
    let mut failed = false;
    for entry in scan(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                stdout.flush()?;
                print_error(err);
                failed = true;
                continue;
            }
        };
        if entry.is_dir() {
            continue;
        }
        let size = entry.apparent_size();
        total += size;
        writeln!(stdout, "{} {}", entry.path().display(), format_size(size))?;
    }
    writeln!(stdout, "\ntotal size: {}", format_size(total))?;
    stdout.flush()?;
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn print_error(msg: impl fmt::Display) {
    let _ = writeln!(io::stderr(), "spacecrab: {}", msg);
}
