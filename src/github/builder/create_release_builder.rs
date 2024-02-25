use super::BuilderExecutor;
use crate::github::{github_client, release::Release, tag::Tag};
use anyhow::Result;

pub struct CreateReleaseBuilder {
    pub owner: String,
    pub repo: String,
    pub release_name: Option<String>,
    pub release_tag: Option<Tag>,
    pub target_branch: Option<String>,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
    pub body: Option<String>,
}

impl CreateReleaseBuilder {
    pub fn new(owner: String, repo: String) -> Self {
        CreateReleaseBuilder {
            owner,
            repo,
            release_name: None,
            release_tag: None,
            target_branch: None,
            draft: None,
            prerelease: None,
            body: None,
        }
    }

    pub fn name<S>(mut self, release_name: S) -> Self
    where
        S: Into<String>,
    {
        self.release_name = Some(release_name.into());
        self
    }

    pub fn tag(mut self, release_tag: &Tag) -> Self {
        self.release_tag = Some(release_tag.to_owned());
        self
    }

    pub fn target_branch<S>(mut self, target_branch: S) -> Self
    where
        S: Into<String>,
    {
        self.target_branch = Some(target_branch.into());
        self
    }

    pub fn draft(mut self, draft: bool) -> Self {
        self.draft = Some(draft);
        self
    }

    pub fn prerelease(mut self, pre_release: bool) -> Self {
        self.prerelease = Some(pre_release);
        self
    }

    pub fn body<S>(mut self, body: S) -> Self
    where
        S: Into<String>,
    {
        self.body = Some(body.into());
        self
    }
}

impl BuilderExecutor for CreateReleaseBuilder {
    type Output = Release;

    async fn execute(self) -> Result<Release> {
        github_client::instance()
            .create_release(
                &self.owner,
                &self.repo,
                &self.release_tag.unwrap(),
                &self.target_branch.unwrap(),
                &self.release_name.unwrap(),
                self.draft.unwrap(),
                self.prerelease.unwrap(),
                &self.body.unwrap_or_default(),
            )
            .await
    }
}
