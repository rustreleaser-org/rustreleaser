use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub owner: String,
    pub repo: String,
    pub target_branch: String,
    pub prerelease: bool,
    pub draft: bool,
    pub name: String,
}
