use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchRefRequest {
    pub r#ref: String,
    pub sha: String,
}

impl BranchRefRequest {
    pub fn new(r#ref: String, sha: String) -> Self {
        Self {
            r#ref: format!("refs/heads/{}", r#ref),
            sha,
        }
    }
}
