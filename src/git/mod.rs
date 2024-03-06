use std::path::PathBuf;

use crate::github::tag::Tag;
use anyhow::{bail, Result};
use git2::Repository;
use itertools::Itertools;
use log::debug;
use semver::Version;

pub fn get_current_tag(base: &PathBuf) -> Result<Tag> {
    let repo = Repository::open(base)?;

    let binding = repo
        .tag_names(None)?
        .into_iter()
        .filter_map(|t| t.map(|t| t.to_string().trim_start_matches("v").to_string()))
        .filter_map(|t| Version::parse(&t).ok())
        .sorted_by(|a, b| a.cmp(b));
    let tag = match binding.last() {
        Some(tag) => tag,
        None => bail!(anyhow::anyhow!("No tags found")),
    };

    debug!("tag: {}", tag);

    Ok(Tag::new(tag.to_string()))
}
