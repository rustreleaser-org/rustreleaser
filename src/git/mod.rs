use anyhow::Result;
use git2::Repository;

pub fn get_current_tag() -> Result<String> {
    let repo = Repository::open(".")?;

    let binding = repo.tag_names(None)?;
    let tag = match binding.into_iter().last().unwrap_or_default() {
        Some(tag) => tag,
        None => {
            // TODO for release, this should be an error
            log::warn!("No tags found");
            ""
        }
    };

    Ok(tag.to_owned())
}
