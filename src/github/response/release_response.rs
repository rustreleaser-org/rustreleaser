use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReleaseResponse {
    pub id: u64,
}
