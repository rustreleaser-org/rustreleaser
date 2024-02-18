use crate::github::{github_client, inner::Inner, release::Release};
use anyhow::Result;

pub struct ReleaseHandler {
    owner: String,
    repo: String,
    release: Release,
}

impl ReleaseHandler {
    pub fn new<S>(owner: S, repo: S, release: Release) -> Self
    where
        S: Into<String>,
    {
        ReleaseHandler {
            owner: owner.into(),
            repo: repo.into(),
            release,
        }
    }

    pub async fn get_by_tag(&self, tag: &str) -> Result<Release> {
        github_client::instance()
            .get_inner()
            .get_release_by_tag(self.owner.to_owned(), self.repo.to_owned(), tag)
            .await
    }
}
