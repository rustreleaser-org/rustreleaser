use super::BrewArch;
use crate::build::os::Os;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Targets(pub Vec<Target>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTarget {
    pub os: Os,
    pub archs: Vec<BrewArch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleTarget {
    pub url: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Single(SingleTarget),
    Multi(MultiTarget),
}
