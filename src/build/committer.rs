#[derive(Debug, Clone)]
pub struct Committer {
    pub author: String,
    pub email: String,
}

impl Default for Committer {
    fn default() -> Self {
        Committer {
            author: "Rafael Vigo".to_string(),
            email: "rvigo07+github@gmail.com".to_string(),
        }
    }
}
