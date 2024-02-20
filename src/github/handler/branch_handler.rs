use crate::github::{builder::upsert_file_builder::UpsertFileBuilder, github_client, sha::Sha};
use anyhow::Result;

pub struct BranchHandler {
    owner: String,
    repo: String,
    branch: String,
}

impl BranchHandler {
    pub fn new<S>(owner: S, repo: S, branch: S) -> Self
    where
        S: Into<String>,
    {
        BranchHandler {
            owner: owner.into(),
            repo: repo.into(),
            branch: branch.into(),
        }
    }

    pub fn upsert_file(&self) -> UpsertFileBuilder {
        UpsertFileBuilder::new(
            self.owner.to_owned(),
            self.repo.to_owned(),
            self.branch.to_owned(),
        )
    }

    pub async fn get_commit_sha(&self) -> Result<Sha> {
        github_client::instance()
            .get_commit_sha(&self.owner, &self.repo, &self.branch)
            .await
    }
}
