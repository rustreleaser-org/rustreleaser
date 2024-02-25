use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReleaseResponse {
    pub id: u64,
}
