use super::{asset::UploadedAsset, generate_checksum_asset};
use crate::{
    brew::package::Package,
    github::{asset::Asset, github_client},
};
use anyhow::{bail, Result};

pub struct Release {
    pub owner: String,
    pub repo: String,
    pub id: u64,
    pub packages: Vec<Package>,
}

impl Release {
    pub fn new<S>(id: u64, owner: S, repo: S) -> Self
    where
        S: Into<String>,
    {
        Release {
            id,
            owner: owner.into(),
            repo: repo.into(),
            packages: vec![],
        }
    }

    pub async fn upload_assets(&self, assets: Vec<Asset>, tag: &str) -> Result<Vec<UploadedAsset>> {
        let mut uploaded = vec![];
        for asset in assets {
            let uploaded_asset = github_client::instance()
                .upload_asset(&asset, &self.owner, tag, &self.repo, self.id)
                .await?;
            log::info!("Uploaded asset: {:#?}", uploaded_asset);
            uploaded.push(uploaded_asset);

            if let Err(err) = self.upload_checksum_asset(&asset, &tag).await {
                bail!(err)
            }
        }

        Ok(uploaded)
    }

    async fn upload_checksum_asset(&self, asset: &Asset, tag: &str) -> Result<()> {
        let checksum_asset = generate_checksum_asset(asset)?;
        let ua = github_client::instance()
            .upload_asset(&checksum_asset, &self.owner, tag, &self.repo, self.id)
            .await?;
        log::info!("Uploaded checksum asset: {:#?}", ua);
        Ok(())
    }
}
