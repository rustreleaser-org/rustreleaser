use anyhow::{bail, Result};
use git2::Repository;

use crate::github::tag::Tag;

pub fn get_current_tag() -> Result<Tag> {
    let repo = Repository::open(".")?;

    let binding = repo.tag_names(None)?;

    let tag = match binding.into_iter().rev().last().unwrap_or_default() {
        Some(tag) => tag,
        None => bail!(anyhow::anyhow!("No tags found")),
    };

    Ok(Tag::new(tag))
}
