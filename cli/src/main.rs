use spacecrab_core::{FilesScanner, Scanner, SizeCalculator, SizeFormatter};
use std::{path::Path, process::ExitCode};

fn main() -> std::io::Result<ExitCode> {
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    let mut total = 0;
    let mut failed = false;
    for path in files {
        match SizeCalculator::size(path.clone()) {
            Ok(size) => {
                total += size;
                println!("{} {}", path.display(), SizeFormatter::format(size))
            }
            Err(err) => {
                eprintln!("spacecrab: {}: {}", path.display(), err);
                failed = true;
            }
        }
    }
    println!("\ntotal size: {}", SizeFormatter::format(total));
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
