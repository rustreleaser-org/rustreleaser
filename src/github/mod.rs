mod arch_os_matrix;
pub mod asset;
pub mod builder;
pub mod github_client;
pub mod handler;
pub mod macros;
pub mod release;
pub mod request;
pub mod response;
pub mod tag;

use self::{
    arch_os_matrix::ArchOsMatrixEntry, asset::UploadedAsset, builder::BuilderExecutor,
    release::Release, tag::Tag,
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

pub async fn release(
    build_info: &Build,
    release_info: &ReleaseConfig,
    base: PathBuf,
    dry_run: bool,
    output_path: &PathBuf,
) -> Result<Vec<Package>> {
    let packages = if build_info.is_multi_target() {
        log::debug!("Running multi target");
        multi(
            build_info.to_owned(),
            release_info.to_owned(),
            base,
            dry_run,
            output_path,
        )
        .await?
    } else {
        log::debug!("Running single target");
        single(
            build_info.to_owned(),
            release_info.to_owned(),
            base,
            dry_run,
            output_path,
        )
        .await?
    };

    Ok(packages)
}

async fn single(
    build_info: Build,
    release_info: ReleaseConfig,
    base: PathBuf,
    dry_run: bool,
    output_path: &PathBuf,
) -> Result<Vec<Package>> {
    // validate binary
    check_binary(&build_info.binary.to_owned(), None, &base)?;

    let tag = git::get_current_tag(&base)?;

    // calculate full binary name
    let binary_name = format!(
        "{}_{}.{}",
        build_info.binary,
        tag.value(),
        build_info.compression.extension()
    );

    log::debug!("binary name: {}", binary_name);

    // zip binary
    log::debug!("zipping binary");
    zip_file(
        &build_info.binary.to_owned(),
        &output_path.join(binary_name.to_owned()),
        base.join(format!("{}/{}", SINGLE_TARGET_DIR, build_info.binary)),
    )?;

    let path = output_path.join(binary_name.to_owned());

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

    if dry_run {
        let package = Package::new(
            asset.name.to_owned(),
            None,
            None,
            None,
            asset.checksum.to_owned().unwrap_or_default(),
        );
        Ok(vec![package])
    } else {
        let release =
            get_release(release_info, &tag, do_create_release, get_release_by_tag).await?;

        // upload to release
        log::debug!("uploading asset");
        let uploaded_assets = match release.upload_assets(vec![asset], &tag, output_path).await {
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
}

async fn multi(
    build_info: Build,
    release_info: ReleaseConfig,
    base: PathBuf,
    dry_run: bool,
    output_path: &PathBuf,
) -> Result<Vec<Package>> {
    let tag = git::get_current_tag(&base)?;

    let archs = build_info.arch.unwrap_or_default();
    let os = build_info.os.unwrap_or_default();
    let mut matrix: Vec<ArchOsMatrixEntry> = Vec::new();

    for arch in &archs {
        for os in &os {
            let binary = build_info.binary.to_owned();
            check_binary(
                &binary,
                Some(format!("{}-{}", &arch.to_string(), &os.to_string())),
                &base,
            )?;

            let mut entry =
                ArchOsMatrixEntry::new(arch, os, binary, tag.value(), &build_info.compression);

            let target = format!("{}-{}", &arch.to_string(), &os.to_string());

            log::debug!("zipping binary for {}", target);

            let entry_name = entry.name.to_owned();

            // zip binary
            zip_file(
                &build_info.binary.to_owned(),
                &output_path.join(entry_name.to_owned()),
                base.join(format!("target/{}/release/{}", target, build_info.binary)),
            )?;

            // create an asset
            let mut asset = Asset::new(entry.name.to_owned(), output_path.join(&entry_name));

            // generate a checksum value
            let checksum = generate_checksum(&asset)
                .unwrap_or_else(|_| panic!("Failed to generate checksum for asset {:#?}", asset));

            // add checksum to asset
            asset.add_checksum(checksum);

            entry.set_asset(asset);
            matrix.push_entry(entry);
        }
    }
    let assets: Vec<Asset> = matrix
        .iter()
        .cloned()
        .filter_map(|entry| entry.asset)
        .collect();
    if dry_run {
        let packages: Vec<Package> = matrix
            .into_iter()
            .map(|entry| {
                let asset = assets
                    .iter()
                    .find(|asset| asset.name == entry.name)
                    .expect("asset not found");
                let _ = generate_checksum_asset(asset, output_path)
                    .expect("failed to generate checksum asset");
                Package::new(
                    asset.name.to_owned(),
                    Some(entry.os.to_owned()),
                    Some(entry.arch.to_owned()),
                    None,
                    asset.checksum.to_owned().unwrap_or_default(),
                )
            })
            .collect();
        Ok(packages)
    } else {
        let release =
            get_release(release_info, &tag, do_create_release, get_release_by_tag).await?;

        // upload to release
        let uploaded_assets = release.upload_assets(assets, &tag, output_path).await?;

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
}

fn zip_file(binary_name: &str, output_path: &PathBuf, binary_path: PathBuf) -> Result<()> {
    let mut file = File::open(binary_path)?;
    let mut archive = Builder::new(Vec::new());

    archive.append_file(binary_name, &mut file)?;

    let compressed_file = File::create(output_path)?;
    let mut encoder = GzEncoder::new(compressed_file, Compression::default());
    encoder.write_all(&archive.into_inner()?)?;

    encoder.finish()?;

    Ok(())
}

fn check_binary(name: &str, target: Option<String>, base: &PathBuf) -> Result<()> {
    log::debug!("checking binary: {} - {:#?}", name, target);
    let binary_path = base.join(if let Some(target) = target {
        format!("target/{}/release/{}", target, name)
    } else {
        format!("target/release/{}", name)
    });

    log::debug!("binary path: {:#?}", binary_path);

    if !PathBuf::from(binary_path).exists() {
        bail!(anyhow::anyhow!(
            "no release folder found, please run `cargo build --release`"
        ));
    }
    Ok(())
}

async fn do_create_release(release_info: ReleaseConfig, tag: &Tag) -> Result<Release> {
    github_client::instance()
        .repo(&release_info.owner, &release_info.repo)
        .releases()
        .create()
        .tag(tag)
        .target_branch(&release_info.target_branch)
        .name(&format!("v{}", tag.value()))
        .draft(release_info.draft)
        .prerelease(release_info.prerelease)
        .body(release_info.body.unwrap_or_default())
        .execute()
        .await
}

async fn get_release_by_tag(release_info: ReleaseConfig, tag: &Tag) -> Result<Release> {
    github_client::instance()
        .repo(&release_info.owner, &release_info.repo)
        .releases()
        .get_by_tag(tag)
        .await
}

async fn get_release<'tag, F, C, FO, CO>(
    release_info: ReleaseConfig,
    tag: &'tag Tag,
    function: F,
    callback: C,
) -> Result<Release>
where
    F: FnOnce(ReleaseConfig, &'tag Tag) -> FO,
    C: FnOnce(ReleaseConfig, &'tag Tag) -> CO,
    FO: Future<Output = Result<Release>>,
    CO: Future<Output = Result<Release>>,
{
    let res = function(release_info.to_owned(), tag).await;
    match res {
        Ok(release) => Ok(release),
        Err(err) => {
            log::warn!(
                "cannot create a release, trying to get the release by tag: {:?}",
                err
            );
            callback(release_info, tag).await
        }
    }
}

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

fn generate_checksum_asset(asset: &Asset, output_path: &PathBuf) -> Result<Asset> {
    if let Some(checksum) = &asset.checksum {
        let sha256_file_name = format!("{}.sha256", asset.name);

        let path = output_path.join(&sha256_file_name);
        fs::write(&path, format!("{}  {}", checksum, asset.name))?;

        let asset = create_asset(&sha256_file_name, path);
        Ok(asset)
    } else {
        bail!(anyhow::anyhow!(
            "checksum is not available for asset {:#?}",
            asset
        ))
    }
}

fn package_asset(asset: &UploadedAsset, os: Option<&Os>, arch: Option<&Arch>) -> Package {
    Package::new(
        asset.name.to_owned(),
        os.map(|os| os.to_owned()),
        arch.map(|arch| arch.to_owned()),
        Some(asset.url.to_owned()),
        asset.checksum.to_owned(),
    )
}
