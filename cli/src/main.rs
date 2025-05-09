use std::path::Path;
use spacecrab_core::{Scanner, FilesScanner, TotalSizeCalculcator};

fn main() -> std::io::Result<()> {
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    let total = TotalSizeCalculcator::total_size(&files).unwrap();
    for path in files {
        println!("{}", path.display())
    }
    println!("total size: {}", total);
    Ok(())
}
