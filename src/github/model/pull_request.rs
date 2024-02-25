use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: u64,
}
