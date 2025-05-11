use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "spacecrab",
    version,
    about = "Scans a directory and reports file sizes"
)]
pub struct Cli {
    /// Directory to scan (defaults to current working directory)
    #[arg(value_name = "DIRECTORY", index = 1, default_value = ".")]
    pub directory: PathBuf,

    /// Recurse into subdirectories
    #[arg(short, long)]
    pub recursive: bool,

    /// Print verbose progress information
    #[arg(short, long)]
    pub verbose: bool,
}