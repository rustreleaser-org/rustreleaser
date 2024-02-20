use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub path: PathBuf,
    pub checksum: Option<String>,
}

impl Asset {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            checksum: None,
        }
    }

    pub fn add_checksum(&mut self, checksum: String) {
        self.checksum = Some(checksum);
    }
}

#[derive(Debug)]
pub struct UploadedAsset {
    pub name: String,
    pub url: String,
    pub checksum: String,
}

impl UploadedAsset {
    pub fn new(name: String, url: String, checksum: String) -> Self {
        Self {
            name,
            url,
            checksum,
        }
    }
}
