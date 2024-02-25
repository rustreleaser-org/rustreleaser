use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "version")]
    name: String,
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Tag { name: name.into() }
    }

    pub fn value(&self) -> &str {
        &self.name
    }

    /// Strip the leading 'v' from the tag name if it exists
    pub fn strip_v_prefix(&self) -> &str {
        if self.name.starts_with('v') {
            self.name.strip_prefix('v').unwrap_or_default()
        } else {
            &self.name
        }
    }
}
