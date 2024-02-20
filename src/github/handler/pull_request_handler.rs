use crate::github::builder::create_pull_request_builder::CreatePullRequestBuilder;

pub struct PullRequestHandler {
    owner: String,
    repo: String,
}

impl PullRequestHandler {
    pub fn new<S>(owner: S, repo: S) -> Self
    where
        S: Into<String>,
    {
        PullRequestHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create(&self) -> CreatePullRequestBuilder {
        CreatePullRequestBuilder::new(self.owner.to_owned(), self.repo.to_owned())
    }
}
