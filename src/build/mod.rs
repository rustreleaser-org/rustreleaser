pub mod arch;
pub mod os;

use arch::Arch;
use os::Os;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub arch: Option<Vec<Arch>>,
    pub os: Option<Vec<Os>>,
    pub binary: String,
    #[serde(default)]
    pub compression: Compression,
}

impl Build {
    pub fn is_multi_target(&self) -> bool {
        self.is_multi_arch() || self.is_multi_os()
    }

    pub fn is_multi_arch(&self) -> bool {
        if let Some(archs) = &self.arch {
            archs.len() > 1
        } else {
            false
        }
    }

    pub fn is_multi_os(&self) -> bool {
        if let Some(oss) = &self.os {
            oss.len() > 1
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Compression {
    #[default]
    TarGz,
}

impl Compression {
    pub fn get_extension(&self) -> &str {
        match self {
            Compression::TarGz => "tar.gz",
        }
    }
}
