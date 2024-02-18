use super::releases_handler::ReleasesHandler;

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

    pub fn releases(&self) -> ReleasesHandler {
        ReleasesHandler::new(self.owner.to_owned(), self.repo.to_owned())
    }
}
