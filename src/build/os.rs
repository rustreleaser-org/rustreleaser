#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Os {
    AppleDarwin,
    UnknownLinuxGnu,
}

impl From<String> for Os {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "apple-darwin" | "darwin" | "macos" => Os::AppleDarwin,
            "unknown-linux-gnu" | "linux" => Os::UnknownLinuxGnu,
            _ => panic!("Unknown arch"),
        }
    }
}

impl ToString for Os {
    fn to_string(&self) -> String {
        match self {
            Os::AppleDarwin => "apple-darwin".to_string(),
            Os::UnknownLinuxGnu => "unknown-linux-gnu".to_string(),
        }
    }
}
