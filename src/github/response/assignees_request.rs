use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssigneesRequest {
    pub assignees: Vec<String>,
}

impl AssigneesRequest {
    pub fn new(assignees: Vec<String>) -> Self {
        Self { assignees }
    }
}
