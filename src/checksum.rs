use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs::File, io, path::Path};

pub fn create<P>(binary_name: &str, path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    log::info!("creating checksum for: {}: {}", binary_name, path.display());

    let mut file = File::open(path)?;

    let mut hasher = Sha256::new();
    let _ = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    let encoded = hex::encode(hash);

    Ok(encoded)
}
