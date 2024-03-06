#[derive(Debug, Clone)]
pub struct Committer {
    pub author: String,
    pub email: String,
}

impl Default for Committer {
    fn default() -> Self {
        Committer {
            author: "rust-releaser".to_string(),
            email: "rust-releaser@github.com".to_string(),
        }
    }
}
