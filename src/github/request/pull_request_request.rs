use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRquestRequest {
    pub title: String,
    pub head: String,
    pub base: String,
    pub body: String,
}

impl PullRquestRequest {
    pub fn new(title: String, head: String, base: String, body: String) -> Self {
        Self {
            title,
            head,
            base,
            body,
        }
    }
}
