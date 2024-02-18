use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Arch {
    Amd64,
    Arm,
    Arm64,
}

impl From<String> for Arch {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "amd64" | "x86_64" => Arch::Amd64,
            "arm" => Arch::Arm,
            "arm64" | "aarch64" => Arch::Arm64,
            _ => panic!("Unknown arch"),
        }
    }
}

impl ToString for Arch {
    fn to_string(&self) -> String {
        match self {
            Arch::Amd64 => "amd64".to_string(),
            Arch::Arm => "arm".to_string(),
            Arch::Arm64 => "aarch64".to_string(),
        }
    }
}
