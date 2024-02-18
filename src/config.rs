use crate::{brew::Brew, build::Build, checksum::Checksum, release::ReleaseConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub build: Build,
    pub brew: Brew,
    #[serde(default)]
    pub checksum: Checksum,
    pub release: ReleaseConfig,
}

impl Config {
    pub async fn load() -> Result<Config> {
        let config_string = tokio::fs::read_to_string("config.yaml").await?;

        let config = serde_yaml::from_str::<Config>(&config_string)?;

        Ok(config)
    }
}
