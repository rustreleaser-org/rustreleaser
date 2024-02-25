use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Sha {
    pub sha: String,
}
