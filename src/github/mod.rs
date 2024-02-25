mod arch_os_matrix;
pub mod asset;
pub mod builder;
pub mod github_client;
pub mod handler;
pub mod model;
pub mod release;

use self::{
    arch_os_matrix::ArchOsMatrixEntry, asset::UploadedAsset, builder::BuilderExecutor,
    release::Release,
};
use crate::{
    brew::package::Package,
    build::{arch::Arch, os::Os, Build},
    checksum,
    config::ReleaseConfig,
    git,
    github::{arch_os_matrix::PushArchOsMatrix, asset::Asset},
};
use anyhow::{bail, Result};
use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
    future::Future,
    io::Write,
    path::{Path, PathBuf},
    vec,
};
use tar::Builder;

const SINGLE_TARGET_DIR: &str = "target/release";
fn create_asset<S, P>(name: S, path: P) -> Asset
where
    S: Into<String>,
    P: AsRef<Path>,
{
    Asset::new(name.into(), path.as_ref().to_path_buf())
}

fn generate_checksum(asset: &Asset) -> Result<String> {
    let checksum = checksum::create(&asset.name, &asset.path)?;
    Ok(checksum)
}

fn generate_checksum_asset(asset: &Asset) -> Result<Asset> {
    if let Some(checksum) = &asset.checksum {
        let sha256_file_name = format!("{}.sha256", asset.name);

        let path = PathBuf::from(&sha256_file_name);
        fs::write(&path, format!("{}  {}", checksum, asset.name))?;

        let asset = create_asset(&sha256_file_name, path);
        Ok(asset)
    } else {
        bail!(anyhow::anyhow!("checksum is not available"))
    }
}

fn package_asset(asset: &UploadedAsset, os: Option<&Os>, arch: Option<&Arch>) -> Package {
    Package::new(
        asset.name.to_owned(),
        os.map(|os| os.to_owned()),
        arch.map(|arch| arch.to_owned()),
        asset.url.to_owned(),
        asset.checksum.to_owned(),
    )
}

pub async fn release(build_info: &Build, release_info: &ReleaseConfig) -> Result<Vec<Package>> {
    let packages = if build_info.is_multi_target() {
        log::debug!("Running multi target");
        multi(build_info.to_owned(), release_info.to_owned()).await?
    } else {
        log::debug!("Running single target");
        single(build_info.to_owned(), release_info.to_owned()).await?
    };

    Ok(packages)
}

async fn single(build_info: Build, release_info: ReleaseConfig) -> Result<Vec<Package>> {
    // validate binary
    check_binary(&build_info.binary.to_owned(), None)?;

    let tag = git::get_current_tag()?;

    // calculate full binary name
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
            tag,
            build_info.compression.get_extension()
        )
    };

    log::debug!("binary name: {}", binary_name);

    // zip binary
    log::debug!("zipping binary");
    zip_file(
        &build_info.binary.to_owned(),
        &binary_name.to_owned(),
        PathBuf::from(format!("{}/{}", SINGLE_TARGET_DIR, build_info.binary)),
    )?;

    let path = PathBuf::from(binary_name.to_owned());

    // create an asset
    log::debug!("creating asset");
    let mut asset = create_asset(binary_name, path);

    // generate a checksum value
    log::debug!("generating checksum");
    let checksum = generate_checksum(&asset)?;

    // add checksum to asset
    log::debug!("adding checksum to asset");
    asset.add_checksum(checksum);

    // create release
    log::debug!("creating release");

    let release = get_release(release_info, &tag, do_create_release, get_release_by_tag).await?;

    // upload to release
    log::debug!("uploading asset");
    let uploaded_assets = match release.upload_assets(vec![asset], &tag).await {
        Ok(uploaded_assets) => uploaded_assets,
        Err(e) => {
            log::error!("Failed to upload asset {:#?}", e);
            bail!(anyhow::anyhow!("Failed to upload asset"))
        }
    };

    // return a package with the asset url and checksum value
    let packages: Vec<Package> = uploaded_assets
        .iter()
        .map(|asset| package_asset(asset, None, None))
        .collect();

    Ok(packages)
}

async fn multi(build_info: Build, release_info: ReleaseConfig) -> Result<Vec<Package>> {
    let tag = git::get_current_tag()?;

    let archs = build_info.arch.unwrap_or_default();
    let os = build_info.os.unwrap_or_default();
    let mut matrix: Vec<ArchOsMatrixEntry> = Vec::new();

    for arch in &archs {
        for os in &os {
            let binary = build_info.binary.to_owned();
            check_binary(
                &binary,
                Some(format!("{}-{}", &arch.to_string(), &os.to_string())),
            )?;

            let mut entry = ArchOsMatrixEntry::new(arch, os, binary, &tag, &build_info.compression);

            let target = format!("{}-{}", &arch.to_string(), &os.to_string());

            log::debug!("zipping binary for {}", target);

            let entry_name = entry.name.to_owned();

            // zip binary
            zip_file(
                &build_info.binary.to_owned(),
                &entry_name,
                PathBuf::from(format!("target/{}/release/{}", target, build_info.binary)),
            )?;

            // create an asset
            let mut asset = Asset::new(entry.name.to_owned(), PathBuf::from(&entry_name));

            // generate a checksum value
            let checksum = generate_checksum(&asset)
                .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

            // add checksum to asset
            asset.add_checksum(checksum);

            entry.set_asset(asset);
            matrix.push_entry(entry);
        }
    }

    let release = get_release(release_info, &tag, do_create_release, get_release_by_tag).await?;

    let assets: Vec<Asset> = matrix
        .iter()
        .cloned()
        .filter_map(|entry| entry.asset)
        .collect();

    // upload to release
    let uploaded_assets = release.upload_assets(assets, &tag).await?;

    let packages: Vec<Package> = matrix
        .into_iter()
        .map(|entry| {
            let asset = uploaded_assets
                .iter()
                .find(|asset| asset.name == entry.name)
                .expect("asset not found");

            package_asset(asset, Some(entry.os), Some(entry.arch))
        })
        .collect();

    Ok(packages)
}

fn zip_file(binary_name: &str, full_binary_name: &str, binary_path: PathBuf) -> Result<()> {
    let mut file = File::open(binary_path)?;
    let mut archive = Builder::new(Vec::new());

    archive.append_file(binary_name, &mut file)?;

    let compressed_file = File::create(full_binary_name)?;
    let mut encoder = GzEncoder::new(compressed_file, Compression::Default);
    encoder.write_all(&archive.into_inner()?)?;

    encoder.finish()?;

    Ok(())
}

fn check_binary(name: &str, target: Option<String>) -> Result<()> {
    log::debug!("checking binary: {} - {:#?}", name, target);
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

async fn do_create_release(release_info: ReleaseConfig, tag: impl Into<String>) -> Result<Release> {
    github_client::instance()
        .repo(&release_info.owner, &release_info.repo)
        .releases()
        .create()
        .tag(tag)
        .target_branch(&release_info.target_branch)
        .name(&release_info.name)
        .draft(release_info.draft)
        .prerelease(release_info.prerelease)
        .execute()
        .await
}

async fn get_release_by_tag(
    release_info: ReleaseConfig,
    tag: impl Into<String>,
) -> Result<Release> {
    github_client::instance()
        .repo(&release_info.owner, &release_info.repo)
        .releases()
        .get_by_tag(&tag.into())
        .await
}

async fn get_release<S, F, C, FO, CO>(
    release_info: ReleaseConfig,
    tag: S,
    function: F,
    callback: C,
) -> Result<Release>
where
    S: Into<String> + Copy,
    F: FnOnce(ReleaseConfig, S) -> FO,
    C: FnOnce(ReleaseConfig, S) -> CO,
    FO: Future<Output = Result<Release>>,
    CO: Future<Output = Result<Release>>,
{
    let res = function(release_info.to_owned(), tag).await;
    match res {
        Ok(release) => Ok(release),
        Err(err) => {
            log::warn!(
                "cannot create a release, trying to get the release by tag: {:#?}",
                err
            );
            callback(release_info, tag).await
        }
    }
}
