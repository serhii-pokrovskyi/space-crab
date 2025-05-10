use std::path::Path;
use spacecrab_core::{
    Scanner,
    FilesScanner,
    SizeCalculator,
    TotalSizeCalculcator,
    SizeFormatter
};

fn main() -> std::io::Result<()> {
    let scaner = FilesScanner;
    let files = scaner.scan(Path::new("."))?;
    let total = TotalSizeCalculcator::total_size(&files).unwrap();
    for path in files {
        let size = SizeCalculator::size(path.clone()).unwrap();
        println!("{} {}", path.display(), SizeFormatter::format(size))
    }
    println!("\ntotal size: {}", SizeFormatter::format(total));
    Ok(())
}
