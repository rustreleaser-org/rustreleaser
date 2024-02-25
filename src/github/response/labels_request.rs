use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelsRequest {
    pub labels: Vec<String>,
}

impl LabelsRequest {
    pub fn new(labels: Vec<String>) -> Self {
        Self { labels }
    }
}
