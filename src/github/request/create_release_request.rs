use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
}

impl CreateReleaseRequest {
    pub fn new(
        tag_name: String,
        target_commitish: String,
        name: String,
        body: String,
        draft: bool,
        prerelease: bool,
    ) -> Self {
        Self {
            tag_name,
            target_commitish,
            name,
            body,
            draft,
            prerelease,
        }
    }
}
