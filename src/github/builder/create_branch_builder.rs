use super::BuilderExecutor;
use crate::github::github_client;
use anyhow::Result;

pub struct CreateBranchBuilder {
    owner: String,
    repo: String,
    branch: String,
    sha: String,
}

impl CreateBranchBuilder {
    pub fn new<S, T>(owner: S, repo: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        CreateBranchBuilder {
            owner: owner.into(),
            repo: repo.into(),
            branch: "".to_string(),
            sha: "".to_string(),
        }
    }

    pub fn branch<S>(mut self, branch: S) -> Self
    where
        S: Into<String>,
    {
        self.branch = branch.into();
        self
    }

    pub fn sha<S>(mut self, sha: S) -> Self
    where
        S: Into<String>,
    {
        self.sha = sha.into();
        self
    }
}

impl BuilderExecutor for CreateBranchBuilder {
    type Output = ();

    async fn execute(self) -> Result<Self::Output> {
        github_client::instance()
            .create_branch(&self.owner, &self.repo, &self.branch, &self.sha)
            .await
    }
}
