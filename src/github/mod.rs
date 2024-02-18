pub mod asset;
pub mod builder;
pub mod github_client;
pub mod handler;
mod inner;
mod release;
mod underlayer;

use self::{handler::HandlerExecutor, release::Release};
use crate::{
    build::Build, checksum, git, github::asset::Asset, release::ReleaseConfig, SINGLE_TARGET_DIR,
};
use anyhow::{bail, Result};
use flate2::{write::GzEncoder, Compression};
use std::{fs::File, io::Write, path::PathBuf};
use tar::Builder;

pub async fn release(build_info: Build, release_info: ReleaseConfig) -> Result<()> {
    if build_info.is_multi_target() {
        multi(build_info, release_info).await?;
    } else {
        log::info!("Running single target");
        single(build_info, release_info).await?;
    }

    Ok(())
}

async fn single(build_info: Build, release_info: ReleaseConfig) -> Result<()> {
    // validate binary
    check_binary(build_info.binary.to_owned(), None)?;

    // calculate full binary name
    let tag = git::get_current_tag()?;

    let binary_name = if tag.is_empty() {
        format!(
            "{}.{}",
            build_info.binary,
            build_info.compression.get_extension()
        )
    } else {
        format!(
            "{}_{}.{}",
            build_info.binary,
            git::get_current_tag()?,
            build_info.compression.get_extension()
        )
    };

    log::info!("binary name: {}", binary_name);

    // zip binary
    zip_file(
        build_info.binary.to_owned(),
        binary_name.to_owned(),
        PathBuf::from(format!("{}/{}", SINGLE_TARGET_DIR, build_info.binary)),
    )?;

    // create an asset
    let asset = Asset::new(
        binary_name.to_owned(),
        PathBuf::from(binary_name.to_owned()),
    );
    let checksum_asset = checksum::Checksum::create(&binary_name, asset.path.to_owned())?;

    log::info!("creating release");

    let release = do_create_release(release_info.clone(), git::get_current_tag()?).await?;

    // upload to release

    log::info!("uploading asset");
    release.upload_assets(vec![asset, checksum_asset]).await;

    Ok(())
}

async fn multi(build_info: Build, release_info: ReleaseConfig) -> Result<()> {
    let archs = build_info.arch.unwrap_or_default();
    let os = build_info.os.unwrap_or_default();
    let mut binary_names: Vec<String> = Vec::new();
    for arch in &archs {
        for os in &os {
            check_binary(
                build_info.binary.clone(),
                Some(format!("{}-{}", &arch.to_string(), &os.to_string())),
            )?;

            binary_names.push(format!(
                "{}_{}_{}_{}.{}",
                build_info.binary,
                git::get_current_tag()?,
                os.to_string(),
                arch.to_string(),
                build_info.compression.get_extension()
            ));
        }
    }

    let assets = binary_names
        .iter()
        .flat_map(|binary| {
            // zip binary
            if let Err(e) = zip_file(
                build_info.binary.to_owned(),
                binary.to_owned(),
                PathBuf::from(format!("target/{}/release/{}", " ", build_info.binary)),
            ) {
                log::error!("failed to zip binary: {}", e);
                bail!(e)
            }

            // create an asset
            Ok(Asset::new(binary.to_owned(), PathBuf::from(binary)))
        })
        .collect::<Vec<_>>();

    let release = do_create_release(release_info.to_owned(), git::get_current_tag()?).await?;

    // upload to release
    release.upload_assets(assets).await;

    Ok(())
}

fn zip_file(binary_name: String, full_binary_name: String, binary_path: PathBuf) -> Result<()> {
    let mut file = File::open(binary_path)?;
    let mut archive = Builder::new(Vec::new());

    archive.append_file(binary_name, &mut file)?;

    let compressed_file = File::create(full_binary_name)?;
    let mut encoder = GzEncoder::new(compressed_file, Compression::Default);
    encoder.write_all(&archive.into_inner()?)?;

    encoder.finish()?;

    Ok(())
}

fn check_binary(name: String, target: Option<String>) -> Result<()> {
    let binary_path = if let Some(target) = target {
        format!("target/{}/release/{}", target, name)
    } else {
        format!("target/release/{}", name)
    };

    if !PathBuf::from(binary_path).exists() {
        let error = "no release folder found, please run `cargo build --release`";
        log::error!("{error}");
        bail!(anyhow::anyhow!(error));
    }
    Ok(())
}

async fn do_create_release(release_info: ReleaseConfig, tag: String) -> Result<Release> {
    // github_client::instance()
    //     .repos(&release_info.owner, &release_info.name)
    //     .releases()
    //     .create()
    //     .tag(tag)
    //     .name(&release_info.name)
    //     .draft(release_info.draft)
    //     .prerelease(release_info.prerelease)
    //     .execute()
    //     .await

    octocrab::instance()
        .repos(&release_info.owner, &release_info.name)
        .releases()
        .get_by_tag("v9.9.9")
        .await
        .map(|release| Release::new(release.id.0, release_info.owner, release_info.name))
        .map_err(|e| anyhow::anyhow!(e))
}
