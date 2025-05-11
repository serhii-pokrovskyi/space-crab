mod cli;

use clap::Parser;
use spacecrab_core::{
    Scanner,
    FilesScanner,
    SizeCalculator,
    TotalSizeCalculcator,
    SizeFormatter
};
use cli::Cli;

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    
    let scaner = FilesScanner;
    let files = scaner.scan(cli.directory.as_path())?;
    let total = TotalSizeCalculcator::total_size(&files).unwrap();
    for path in files {
        let size = SizeCalculator::size(path.clone()).unwrap();
        println!("{} {}", path.display(), SizeFormatter::format(size))
    }
    println!("\ntotal size: {}", SizeFormatter::format(total));
    Ok(())
}
