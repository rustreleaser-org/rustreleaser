use super::BuilderExecutor;
use crate::github::{builder::create_pull_request_builder::Commiter, github_client};

pub struct UpsertFileBuilder {
    owner: String,
    repo: String,
    path: String,
    commit_message: String,
    content: String,
    commiter: Commiter,
    head: String,
}

impl UpsertFileBuilder {
    pub fn new<S, T>(owner: S, repo: T, branch: S) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        UpsertFileBuilder {
            owner: owner.into(),
            repo: repo.into(),
            path: "".to_string(),
            commit_message: "".to_string(),
            content: "".to_string(),
            commiter: Commiter::default(),
            head: branch.into(),
        }
    }

    pub fn path<S>(mut self, path: S) -> Self
    where
        S: Into<String>,
    {
        self.path = path.into();
        self
    }

    pub fn message<S>(mut self, message: S) -> Self
    where
        S: Into<String>,
    {
        self.commit_message = message.into();
        self
    }

    pub fn content<S>(mut self, content: S) -> Self
    where
        S: Into<String>,
    {
        self.content = content.into();
        self
    }

    pub fn commiter(mut self, commiter: &Commiter) -> Self {
        self.commiter = commiter.to_owned();
        self
    }
}

impl BuilderExecutor for UpsertFileBuilder {
    type Output = ();

    async fn execute(self) -> anyhow::Result<Self::Output> {
        github_client::instance()
            .update_file(
                &self.owner,
                &self.repo,
                &self.path,
                &self.content,
                self.commit_message,
                self.commiter,
                self.head,
            )
            .await
    }
}
