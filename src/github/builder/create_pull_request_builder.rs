use super::BuilderExecutor;
use crate::github::{github_client, model::pull_request::PullRequest};

pub struct CreatePullRequestBuilder {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub commiter: Option<Commiter>,
    pub base: Option<String>,
    pub head: Option<String>,
}

#[derive(Clone)]
pub struct Commiter {
    pub author: String,
    pub email: String,
}

impl Default for Commiter {
    fn default() -> Self {
        Commiter {
            author: "Rafael Vigo".to_string(),
            email: "rvigo07+github@gmail.com".to_string(),
        }
    }
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
            commiter: None,
            base: None,
            head: None,
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

    pub fn commiter(mut self, commiter: &Commiter) -> Self {
        self.commiter = Some(commiter.to_owned());
        self
    }

    pub fn base<S>(mut self, base: S) -> Self
    where
        S: Into<String>,
    {
        self.base = Some(base.into());
        self
    }

    pub fn head<S>(mut self, head: S) -> Self
    where
        S: Into<String>,
    {
        self.head = Some(head.into());
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
                &self.head.unwrap(),
                &self.base.unwrap(),
                self.body,
                self.assignees.unwrap_or_default(),
                self.labels.unwrap_or_default(),
            )
            .await
    }
}
