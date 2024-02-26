use crate::{
    brew::{install::Install, repository::Repository},
    build::Build,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

const MAIN_BRANCH_NAME: &str = "main";
const BREW_DEFAULT_COMMIT_MESSAGE: &str = "update formula";

const PR_DEFAULT_BASE_BRANCH_NAME: &str = MAIN_BRANCH_NAME;
const PR_DEFAULT_HEAD_BRANCH_NAME: &str = "bumps-formula-version";

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
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: String,
    pub install: Install,
    #[serde(default)]
    pub license: String,
    #[serde(default = "BrewConfig::main_branch_name")]
    pub head: String,
    #[serde(default)]
    pub test: String,
    #[serde(default)]
    pub caveats: String,
    #[serde(default = "BrewConfig::default_commit_message")]
    pub commit_message: String,
    pub commit_author: Option<CommitterConfig>,
    pub pull_request: Option<PullRequestConfig>,
    pub repository: Repository,
}

impl BrewConfig {
    fn main_branch_name() -> String {
        MAIN_BRANCH_NAME.to_owned()
    }

    fn default_commit_message() -> String {
        BREW_DEFAULT_COMMIT_MESSAGE.to_owned()
    }
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
    #[serde(default)]
    pub draft: bool,
    #[serde(default = "PullRequestConfig::default_base_branch_name")]
    pub base: String,
    #[serde(default = "PullRequestConfig::default_head_branch_name")]
    pub head: String,
}

impl PullRequestConfig {
    fn default_base_branch_name() -> String {
        PR_DEFAULT_BASE_BRANCH_NAME.to_owned()
    }

    fn default_head_branch_name() -> String {
        PR_DEFAULT_HEAD_BRANCH_NAME.to_owned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    pub owner: String,
    pub repo: String,
    pub target_branch: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    pub name: String,
    pub body: Option<String>,
}
