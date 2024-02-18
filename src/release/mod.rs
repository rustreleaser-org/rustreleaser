#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename(serialize = "release"))]
pub struct ReleaseConfig {
    pub owner: String,
    pub name: String,
    pub prerelease: bool,
    pub draft: bool,
}

pub enum Target {
    Single,
    Multi(Vec<String>),
}
