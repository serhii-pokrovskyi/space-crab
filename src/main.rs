mod path_scaner;

use std::path::{self, Path};
use path_scaner::{Scanner, FilesScaner};

fn main() -> std::io::Result<()> {
    let scaner = FilesScaner;
    let files = scaner.scan(Path::new("."))?;
    for path in files {
        println!("{}", path.display())
    }
    Ok(())
}
