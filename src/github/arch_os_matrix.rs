use super::asset::Asset;
use crate::build::{arch::Arch, compression::Compression, os::Os};

#[derive(Debug, Clone)]
pub struct ArchOsMatrixEntry<'matrix> {
    pub arch: &'matrix Arch,
    pub os: &'matrix Os,
    pub name: String,
    pub asset: Option<Asset>,
}

impl<'matrix> ArchOsMatrixEntry<'matrix> {
    pub fn new(
        arch: &'matrix Arch,
        os: &'matrix Os,
        name: String,
        tag: &'matrix str,
        compression: &'matrix Compression,
    ) -> Self {
        let name = format!(
            "{}_{}_{}_{}.{}",
            name,
            tag,
            arch.to_string(),
            os.to_string(),
            compression.extension()
        );
        Self {
            arch,
            os,
            name,
            asset: None,
        }
    }

    pub fn set_asset(&mut self, asset: Asset) {
        self.asset = Some(asset);
    }
}

pub trait PushArchOsMatrix<'matrix> {
    fn push_entry(&mut self, entry: ArchOsMatrixEntry<'matrix>);
}

impl<'matrix> PushArchOsMatrix<'matrix> for Vec<ArchOsMatrixEntry<'matrix>> {
    fn push_entry(&mut self, entry: ArchOsMatrixEntry<'matrix>) {
        self.push(entry);
    }
}
