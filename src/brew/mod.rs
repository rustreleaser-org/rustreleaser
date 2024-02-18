use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallInfo {
    pub bin_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brew {
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub install_info: String,
    pub repository: Repository,
}
