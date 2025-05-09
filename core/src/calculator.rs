use std::{
    fs,
    io,
    path::{PathBuf}
};

pub struct TotalSizeCalculcator;

impl TotalSizeCalculcator {
    pub fn total_size(bufs: &Vec<PathBuf>) -> io::Result<u64> {
        let mut total = 0;
        for path in bufs {
            total += fs::metadata(path)?.len();
        }
        Ok(total)
    }
}