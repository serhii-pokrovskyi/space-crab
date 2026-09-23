use spacecrab_core::{FilesScanner, Scanner, SizeCalculator, SizeFormatter};
use std::{
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

fn main() -> io::Result<ExitCode> {
    match run() {
        // The reader went away (e.g. `spacecrab | head`): nothing left to print to.
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(ExitCode::SUCCESS),
        result => result,
    }
}

fn run() -> io::Result<ExitCode> {
    let scanner = FilesScanner;
    let files = scanner.scan(Path::new("."))?;
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut total = 0;
    let mut failed = false;
    for path in files {
        match SizeCalculator::size(&path) {
            Ok(size) => {
                total += size;
                writeln!(stdout, "{} {}", path.display(), SizeFormatter::format(size))?;
            }
            Err(err) => {
                stdout.flush()?;
                eprintln!("spacecrab: {}: {}", path.display(), err);
                failed = true;
            }
        }
    }
    writeln!(stdout, "\ntotal size: {}", SizeFormatter::format(total))?;
    stdout.flush()?;
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
