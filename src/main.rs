mod brew;
mod build;
mod checksum;
mod cli;
mod config;
mod git;
mod github;
mod http;
mod logger;
mod template;

use tokio::process::Command;

use crate::{cli::Opts, template::Template};
use anyhow::Result;
use clap::Parser;
use config::ReleaserConfig;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let opts = Opts::parse();

    log::info!("Starting");
    let config = ReleaserConfig::load(opts.config).await?;

    if !std::fs::metadata(&opts.output).is_ok() {
        log::info!("Creating directory: {:?}", &opts.output);
        std::fs::create_dir_all(&opts.output)?;
    }

    let build_info = config.build;

    log::info!("Building with {:?}", build_info.tool);

    build::build(&build_info, opts.path.clone(), opts.dry_run).await?;

    let release_info = config.release;

    log::info!("Creating release");
    let packages = github::release(
        &build_info,
        &release_info,
        opts.path.clone(),
        opts.dry_run,
        &opts.output,
    )
    .await?;

    if config.brew.is_some() {
        log::info!("Creating brew formula");
        brew::release(
            config.brew.unwrap(),
            packages,
            Template::from(build_info),
            opts.path.clone(),
            opts.dry_run,
            &opts.output,
        )
        .await?;
    }

    if config.crates_io.is_some() && !opts.dry_run {
        let crates_io = config.crates_io.unwrap();
        for package in &crates_io.packages {
            log::info!("Publishing {} to crates.io", package);
            let mut cmd = Command::new("cargo");
            cmd.arg("publish").current_dir(opts.path.clone());
            if crates_io.allow_dirty.unwrap_or(false) {
                cmd.arg("--allow-dirty");
            }
            if crates_io.no_verify.unwrap_or(false) {
                cmd.arg("--no-verify");
            }
            if crates_io.clone().registry.is_some() {
                cmd.arg("--registry")
                    .arg(crates_io.clone().registry.unwrap());
            }
            if crates_io.clone().index.is_some() {
                cmd.arg("--index").arg(crates_io.clone().index.unwrap());
            }
            cmd.arg("--package").arg(package);

            cmd.status().await?;
        }
    }

    Ok(())
}
