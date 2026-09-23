use spacecrab_core::{file_size, format_size, scan};
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
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut total = 0;
    let mut failed = false;
    for entry in scan(Path::new(".")) {
        let path = match entry {
            Ok(path) => path,
            Err(err) => {
                stdout.flush()?;
                eprintln!("spacecrab: {}", err);
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
                eprintln!("spacecrab: {}: {}", path.display(), err);
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
