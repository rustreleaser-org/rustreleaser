use crate::{
    brew::{install::Install, repository::Repository},
    build::Build,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub build: Build,
    pub brew: Option<BrewConfig>,
    pub release: ReleaseConfig,
}

impl Config {
    pub async fn load() -> Result<Config> {
        let config_string = tokio::fs::read_to_string("config.yaml").await?;

        let config = serde_yaml::from_str::<Config>(&config_string)?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewConfig {
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub install: Install,
    pub repository: Repository,
    pub license: Option<String>,
    pub head: Option<String>,
    pub test: Option<String>,
    pub caveats: Option<String>,
    pub commit_message: Option<String>,
    pub commit_author: Option<CommitterConfig>,
    pub pull_request: Option<PullRequestConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitterConfig {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestConfig {
    pub title: Option<String>,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub milestone: Option<u64>,
    pub draft: Option<bool>,
    pub base: Option<String>,
    pub head: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub owner: String,
    pub repo: String,
    pub target_branch: String,
    pub prerelease: bool,
    pub draft: bool,
    pub name: String,
}
