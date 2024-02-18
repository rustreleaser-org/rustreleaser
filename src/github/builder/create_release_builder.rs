use crate::github::{github_client, handler::HandlerExecutor, inner::Inner, release::Release};
use anyhow::Result;

pub struct CreateReleaseBuilder {
    pub owner: String,
    pub repo: String,
    pub release_name: Option<String>,
    pub release_tag: Option<String>,
    pub target_branch: Option<String>,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
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
        }
    }

    pub fn name<S>(mut self, release_name: S) -> Self
    where
        S: Into<String>,
    {
        self.release_name = Some(release_name.into());
        self
    }

    pub fn tag<S>(mut self, release_tag: S) -> Self
    where
        S: Into<String>,
    {
        self.release_tag = Some(release_tag.into());
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
}

impl HandlerExecutor for CreateReleaseBuilder {
    type Output = Release;

    async fn execute(self) -> Result<Release> {
        github_client::instance()
            .get_inner()
            .create_release(self)
            .await
    }
}
