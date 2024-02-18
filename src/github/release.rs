use crate::github::{asset::Asset, github_client};

pub struct Release {
    pub owner: String,
    pub repo: String,
    pub id: u64,
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
        }
    }

    pub async fn upload_asset(&self, asset: Asset) -> &Release {
        if (github_client::instance()
            .upload_asset(asset, &self.owner, &self.repo, self.id)
            .await)
            .is_ok()
        {
            return self;
        }

        panic!("Failed to upload asset");
    }

    pub async fn upload_assets(&self, assets: Vec<Asset>) -> &Release {
        for asset in assets {
            self.upload_asset(asset).await;
        }

        self
    }
}
