#![deny(clippy::print_stderr, clippy::print_stdout)]

use spacecrab_core::{file_size, format_size, scan};
use std::{
    fmt,
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            print_error(format_args!("error writing output: {}", err));
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<ExitCode> {
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut total = 0;
    let mut failed = false;
    for entry in scan(Path::new(".")) {
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
