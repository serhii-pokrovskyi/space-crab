mod path_scanner;

use std::path::Path;
use path_scanner::{Scanner, FilesScanner};

fn main() -> std::io::Result<()> {
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    for path in files {
        println!("{}", path.display())
    }
    Ok(())
}
