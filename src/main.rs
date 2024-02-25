mod brew;
mod build;
mod checksum;
mod config;
pub mod git;
mod github;
mod http;
mod logger;
mod release;
mod template;

use anyhow::Result;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    log::info!("Starting");

    let config = Config::load().await?;

    let build_info = config.build;
    let release_info = config.release;

    log::info!("Creating release");
    let packages = github::release(&build_info, &release_info).await?;

    if config.brew.is_some() {
        log::info!("Creating brew formula");
        brew::release(config.brew.unwrap(), packages, build_info.is_multi_target()).await?;
    }

    Ok(())
}
