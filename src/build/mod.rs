pub mod arch;
pub mod committer;
pub mod compression;
pub mod os;

use std::path::PathBuf;

use self::compression::Compression;
use anyhow::Result;
use arch::Arch;
use os::Os;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub arch: Option<Vec<Arch>>,
    pub os: Option<Vec<Os>>,
    pub binary: String,
    #[serde(default)]
    pub compression: Compression,
    #[serde(default)]
    pub tool: Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Tool {
    #[serde(rename = "cargo")]
    #[default]
    Cargo,
    #[serde(rename = "cross")]
    Cross,
}

impl Build {
    pub fn is_multi_target(&self) -> bool {
        self.is_multi_arch() || self.is_multi_os()
    }

    pub fn is_multi_arch(&self) -> bool {
        if let Some(archs) = &self.arch {
            !archs.is_empty()
        } else {
            false
        }
    }

    pub fn is_multi_os(&self) -> bool {
        if let Some(oss) = &self.os {
            !oss.is_empty()
        } else {
            false
        }
    }
}

pub async fn build(build_info: &Build, path: PathBuf, dry_run: bool) -> Result<()> {
    if build_info.is_multi_target() {
        log::info!("Building for multiple targets");
        for arch in build_info.arch.as_ref().unwrap_or(&vec![]) {
            for os in build_info.os.as_ref().unwrap_or(&vec![]) {
                build_target(&build_info, &path, &arch, &os, dry_run).await?;
            }
        }
    } else {
        log::info!("Building for single target");
        build_target(
            &build_info,
            &path,
            &Arch::current(),
            &Os::current(),
            dry_run,
        )
        .await?;
    }

    Ok(())
}

async fn build_target(
    build_info: &Build,
    path: &PathBuf,
    arch: &Arch,
    os: &Os,
    dry_run: bool,
) -> Result<()> {
    let toolchain = os_arch_to_toolchain(os, arch);
    let mut cmd = match &build_info.tool {
        Tool::Cargo => {
            let mut cmd = tokio::process::Command::new("cargo");
            cmd.arg("build");
            cmd.arg("--release");
            cmd.arg("--target").arg(toolchain);
            cmd.current_dir(path);
            cmd
        }
        Tool::Cross => {
            let mut cmd = tokio::process::Command::new("cross");
            cmd.arg("build");
            cmd.arg("--release");
            cmd.arg("--target").arg(toolchain);
            cmd.current_dir(path);
            cmd
        }
    };

    if dry_run {
        log::info!("Would run: {:?}", cmd);
    } else {
        cmd.status().await?;
    }

    Ok(())
}

fn os_arch_to_toolchain(os: &Os, arch: &Arch) -> String {
    format!(
        "{}-{}",
        match arch {
            Arch::Amd64 => "x86_64",
            Arch::Arm => "arm",
            Arch::Arm64 => "aarch64",
        },
        match os {
            Os::UnknownLinuxGnu => "unknown-linux-gnu",
            Os::AppleDarwin => "apple-darwin",
        },
    )
}
