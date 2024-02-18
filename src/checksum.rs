use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self},
    path::PathBuf,
};

use crate::github::asset::Asset;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub enum ChecksumAlgorithm {
    #[default]
    SHA256,
    MD5,
}

const DEFAULT_CHECKSUM_FILE_NAME: &str = "checksum.txt";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Checksum {
    #[serde(default = "default_checksum_file_name")]
    pub file_name: String,
    #[serde(default)]
    pub algorithm: ChecksumAlgorithm,
}

fn default_checksum_file_name() -> String {
    DEFAULT_CHECKSUM_FILE_NAME.to_owned()
}

impl Default for Checksum {
    fn default() -> Self {
        Self {
            file_name: DEFAULT_CHECKSUM_FILE_NAME.to_owned(),
            algorithm: Default::default(),
        }
    }
}

impl Checksum {
    pub fn create(binary_name: &str, path: PathBuf) -> Result<Asset> {
        log::info!("creating checksum for: {}: {}", binary_name, path.display());

        let mut file = File::open(&path)?;

        let mut hasher = Sha256::new();
        let _ = io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();

        let encoded = hex::encode(hash);

        let sha256_file_name = format!("{}.sha256", binary_name);

        fs::write(
            sha256_file_name.clone(),
            format!("{}  {}", encoded, binary_name),
        )?;

        let asset = Asset::new(
            sha256_file_name.to_owned(),
            PathBuf::from(sha256_file_name.to_owned()),
        );

        Ok(asset)
    }
}
