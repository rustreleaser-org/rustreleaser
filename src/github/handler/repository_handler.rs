use super::{
    branch_handler::BranchHandler, branches_handler::BranchesHandler,
    pull_request_handler::PullRequestHandler, release_handler::ReleaseHandler,
};

pub struct RepositoryHandler {
    owner: String,
    repo: String,
}

impl RepositoryHandler {
    pub fn new<S>(owner: S, repo: S) -> Self
    where
        S: Into<String>,
    {
        RepositoryHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn releases(&self) -> ReleaseHandler {
        ReleaseHandler::new(self.owner.to_owned(), self.repo.to_owned())
    }

    pub fn branches(&self) -> BranchesHandler {
        BranchesHandler::new(self.owner.to_owned(), self.repo.to_owned())
    }

    pub fn branch(&self, branch: &str) -> BranchHandler {
        BranchHandler::new(
            self.owner.to_owned(),
            self.repo.to_owned(),
            branch.to_owned(),
        )
    }

    pub fn pull_request(&self) -> PullRequestHandler {
        PullRequestHandler::new(self.owner.to_owned(), self.repo.to_owned())
    }
}
