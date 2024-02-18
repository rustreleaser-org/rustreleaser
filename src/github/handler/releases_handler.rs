use crate::github::{
    builder::create_release_builder::CreateReleaseBuilder,
    github_client::{self},
    inner::Inner,
    release::Release,
};
use anyhow::Result;

pub struct ReleasesHandler {
    owner: String,
    repo: String,
}

impl ReleasesHandler {
    pub fn new<S>(owner: S, repo: S) -> Self
    where
        S: Into<String>,
    {
        ReleasesHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create(&self) -> CreateReleaseBuilder {
        CreateReleaseBuilder::new(self.owner.to_owned(), self.repo.to_owned())
    }

    pub async fn get_by_tag(&self, tag: &str) -> Result<Release> {
        // self.client
        github_client::instance()
            .get_inner()
            .get_release_by_tag(self.owner.to_owned(), self.repo.to_owned(), tag)
            .await
    }
}
