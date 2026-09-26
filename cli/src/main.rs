#![deny(clippy::print_stderr, clippy::print_stdout)]

use clap::Parser;
use spacecrab_core::{file_size, format_size, scan};
use std::{
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(version, about)]
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
        let path = match entry {
            Ok(path) => path,
            Err(err) => {
                stdout.flush()?;
                print_error(err);
                failed = true;
                continue;
            }
        };
        match file_size(&path) {
            Ok(size) => {
                total += size;
                writeln!(stdout, "{} {}", path.display(), format_size(size))?;
            }
            Err(err) => {
                stdout.flush()?;
                print_error(format_args!("{}: {}", path.display(), err));
                failed = true;
            }
        }
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
