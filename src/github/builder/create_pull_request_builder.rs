use super::BuilderExecutor;
use crate::{
    build::committer::Committer,
    github::{github_client, response::pull_request_response::PullRequest},
};

pub struct CreatePullRequestBuilder {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub committer: Option<Committer>,
    pub base: String,
    pub head: String,
}

impl CreatePullRequestBuilder {
    pub fn new<S>(owner: S, repo: S) -> Self
    where
        S: Into<String>,
    {
        CreatePullRequestBuilder {
            owner: owner.into(),
            repo: repo.into(),
            title: String::new(),
            body: None,
            labels: None,
            assignees: None,
            committer: None,
            base: String::new(),
            head: String::new(),
        }
    }

    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn body<S>(mut self, body: S) -> Self
    where
        S: Into<String>,
    {
        self.body = Some(body.into());
        self
    }

    pub fn labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    pub fn assignees(mut self, assignees: Vec<String>) -> Self {
        self.assignees = Some(assignees);
        self
    }

    pub fn committer(mut self, committer: &Committer) -> Self {
        self.committer = Some(committer.to_owned());
        self
    }

    pub fn base<S>(mut self, base: S) -> Self
    where
        S: Into<String>,
    {
        self.base = base.into();
        self
    }

    pub fn head<S>(mut self, head: S) -> Self
    where
        S: Into<String>,
    {
        self.head = head.into();
        self
    }
}

impl BuilderExecutor for CreatePullRequestBuilder {
    type Output = PullRequest;

    async fn execute(self) -> anyhow::Result<Self::Output> {
        github_client::instance()
            .create_pull_request(
                &self.owner,
                &self.repo,
                &self.title,
                &self.head,
                &self.base,
                &self.body.unwrap_or_default(),
                self.assignees.unwrap_or_default(),
                self.labels.unwrap_or_default(),
            )
            .await
    }
}
