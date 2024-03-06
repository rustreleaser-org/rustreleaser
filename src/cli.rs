use std::path::PathBuf;

use clap::Parser;
#[derive(Parser, Debug)]
#[clap(about, author, version, name = "rust-releaser")]
pub struct Opts {
    /// Path to the project
    #[clap(default_value = ".")]
    pub path: PathBuf,
    /// Path to the config file
    #[clap(short, long, default_value = "releaser.toml")]
    pub config: String,
    /// Dry run (do not upload anything)
    #[clap(short, long)]
    pub dry_run: bool,
    /// Output directory for temporary files
    #[clap(short, long, default_value = ".")]
    pub output: PathBuf,
}
