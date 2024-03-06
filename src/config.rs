use crate::{
    brew::{install::Install, repository::Repository},
    build::Build,
};
use anyhow::Result;
use config::Config;
use serde::{Deserialize, Serialize};
const MAIN_BRANCH_NAME: &str = "main";
const BREW_DEFAULT_COMMIT_MESSAGE: &str = "update formula";

const PR_DEFAULT_BASE_BRANCH_NAME: &str = MAIN_BRANCH_NAME;
const PR_DEFAULT_HEAD_BRANCH_NAME: &str = "bumps-formula-version";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaserConfig {
    pub build: Build,
    pub brew: Option<BrewConfig>,
    pub release: ReleaseConfig,
    pub crates_io: Option<CratesIoConfig>,
}

impl ReleaserConfig {
    pub async fn load(path: String) -> Result<Self> {
        let config = Config::builder()
            .add_source(config::File::with_name(&path))
            .add_source(config::Environment::with_prefix("RELEASER"))
            .build()?
            .try_deserialize()?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CratesIoConfig {
    pub packages: Vec<String>,
    pub registry: Option<String>,
    pub index: Option<String>,
    pub allow_dirty: Option<bool>,
    pub no_verify: Option<bool>,
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
    pub path: Option<String>,
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
    pub body: Option<String>,
}
