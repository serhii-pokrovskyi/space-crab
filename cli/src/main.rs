use spacecrab_core::{FilesScanner, Scanner, SizeCalculator, SizeFormatter};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    let mut total = 0;
    for path in files {
        let size = SizeCalculator::size(path.clone()).unwrap();
        total += size;
        println!("{} {}", path.display(), SizeFormatter::format(size))
    }
    println!("\ntotal size: {}", SizeFormatter::format(total));
    Ok(())
}
