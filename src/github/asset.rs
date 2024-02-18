use std::path::PathBuf;

pub struct Asset {
    pub name: String,
    pub path: PathBuf,
}

impl Asset {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}
