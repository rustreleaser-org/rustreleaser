use crate::build::{arch::Arch, os::Os};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub os: Option<Os>,
    pub arch: Option<Arch>,
    pub url: String,
    pub sha256: String,
}

impl Package {
    pub fn new(
        name: String,
        os: Option<Os>,
        arch: Option<Arch>,
        url: String,
        sha256: String,
    ) -> Self {
        Self {
            name,
            os,
            arch,
            url,
            sha256,
        }
    }
}
