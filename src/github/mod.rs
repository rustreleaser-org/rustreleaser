mod arch_os_matrix;
pub mod asset;
pub mod builder;
pub mod github_client;
pub mod handler;
mod inner;
pub mod pull_request;
mod release;
pub mod sha;
mod underlayer;

use self::{
    arch_os_matrix::ArchOsMatrixEntry, asset::UploadedAsset, builder::BuilderExecutor,
    release::Release,
};
use crate::{
    brew::package::Package,
    build::{arch::Arch, os::Os, Build},
    checksum, git,
    github::{arch_os_matrix::PushArchOsMatrix, asset::Asset},
    release::ReleaseConfig,
};
use anyhow::{bail, Result};
use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
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

pub async fn release(build_info: Build, release_info: ReleaseConfig) -> Result<Vec<Package>> {
    let packages = if build_info.is_multi_target() {
        log::info!("Running multi target");
        multi(build_info, release_info).await?
    } else {
        log::info!("Running single target");
        single(build_info, release_info).await?
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
    log::info!("zipping binary");
    zip_file(
        &build_info.binary.to_owned(),
        &binary_name.to_owned(),
        PathBuf::from(format!("{}/{}", SINGLE_TARGET_DIR, build_info.binary)),
    )?;

    let path = PathBuf::from(binary_name.to_owned());

    // create an asset
    log::info!("creating asset");
    let mut asset = create_asset(binary_name, path);

    // generate a checksum value
    log::info!("generating checksum");
    let checksum = generate_checksum(&asset)?;

    // add checksum to asset
    log::info!("adding checksum to asset");
    asset.add_checksum(checksum);

    // create release
    log::info!("creating release");
    // TODO improve this flow
    let release = match do_create_release(release_info.to_owned(), &tag).await {
        Ok(release) => release,
        Err(e) => {
            log::error!("Failed to create release {:#?}", e.root_cause());
            log::info!("Trying to get release by tag");
            match get_release_by_tag(release_info, &tag).await {
                Ok(release) => release,
                Err(e) => {
                    log::error!("Failed to get release by tag {:#?}", e.root_cause());
                    bail!(anyhow::anyhow!("Failed to create or get release"))
                }
            }
        }
    };

    // upload to release
    log::info!("uploading asset");
    let uploaded_assets = match release.upload_assets(vec![asset], &tag).await {
        Ok(uploaded_assets) => uploaded_assets,
        Err(e) => {
            log::error!("Failed to upload asset {:#?}", e.root_cause());
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
    // FIXME
    let tag = if git::get_current_tag()?.is_empty() {
        "v9.9.9".to_owned()
    } else {
        git::get_current_tag()?
    };

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

            log::info!("zipping binary for {}", target);

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
                .expect(format!("Failed to generate checksum for asset {:#?}", asset).as_str());

            // add checksum to asset
            asset.add_checksum(checksum);

            entry.set_asset(asset);
            matrix.push_entry(entry);
        }
    }

    let release = match do_create_release(release_info.to_owned(), &tag).await {
        Ok(release) => release,
        Err(e) => {
            log::error!("Failed to create release {:#?}", e.root_cause());
            log::info!("Trying to get release by tag");
            get_release_by_tag(release_info, &tag).await?
        }
    };

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
    log::info!("checking binary: {} - {:#?}", name, target);
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
        .name(&release_info.repo)
        .draft(release_info.draft)
        .prerelease(release_info.prerelease)
        .execute()
        .await
}

async fn get_release_by_tag(release_info: ReleaseConfig, tag: &str) -> Result<Release> {
    github_client::instance()
        .repo(&release_info.owner, &release_info.repo)
        .releases()
        .get_by_tag(tag)
        .await
}
