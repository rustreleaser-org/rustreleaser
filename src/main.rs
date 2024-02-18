mod brew;
mod build;
mod checksum;
mod config;
pub mod git;
mod github;
mod http;
mod logger;
mod release;
mod templating;

use anyhow::Result;
use config::Config;

const SINGLE_TARGET_DIR: &str = "target/release";

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    log::info!("Starting...");

    let config = Config::load().await?;

    let build_info = config.build;
    let release_info = config.release;

    github::release(build_info, release_info).await?;

    Ok(())
}
