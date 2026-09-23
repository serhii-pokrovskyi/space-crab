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
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    let mut stdout = io::stdout();
    let mut total = 0;
    let mut failed = false;
    for path in files {
        match SizeCalculator::size(path.clone()) {
            Ok(size) => {
                total += size;
                writeln!(stdout, "{} {}", path.display(), SizeFormatter::format(size))?;
            }
            Err(err) => {
                eprintln!("spacecrab: {}: {}", path.display(), err);
                failed = true;
            }
        }
    }
    writeln!(stdout, "\ntotal size: {}", SizeFormatter::format(total))?;
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
