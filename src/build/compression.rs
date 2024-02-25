use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Compression {
    #[default]
    TarGz,
}

impl Compression {
    pub fn extension(&self) -> &str {
        match self {
            Compression::TarGz => "tar.gz",
        }
    }
}
