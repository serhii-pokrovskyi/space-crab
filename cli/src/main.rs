use std::path::Path;
use spacecrab_core::{Scanner, FilesScanner};

fn main() -> std::io::Result<()> {
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    for path in files {
        println!("{}", path.display())
    }
    Ok(())
}
