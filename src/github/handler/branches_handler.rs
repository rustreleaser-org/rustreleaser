use crate::github::builder::create_branch_builder::CreateBranchBuilder;

pub struct BranchesHandler {
    owner: String,
    repo: String,
}

impl BranchesHandler {
    pub fn new<S>(owner: S, repo: S) -> Self
    where
        S: Into<String>,
    {
        BranchesHandler {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn create(&self) -> CreateBranchBuilder {
        CreateBranchBuilder::new(self.owner.to_owned(), self.repo.to_owned())
    }
}
