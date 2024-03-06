pub mod install;
pub mod package;
pub mod repository;
pub mod target;

use self::{
    install::Install,
    package::Package,
    repository::Repository,
    target::{MultiTarget, SingleTarget, Target, Targets},
};
use crate::{
    build::{arch::Arch, committer::Committer},
    config::{BrewConfig, CommitterConfig, PullRequestConfig},
    git,
    github::{builder::BuilderExecutor, github_client, tag::Tag},
    template::{handlebars, Template},
};
use anyhow::{Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brew {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub head: String,
    pub test: String,
    pub caveats: String,
    pub commit_message: String,
    pub commit_author: Option<CommitterConfig>,
    pub install_info: Install,
    pub repository: Repository,
    #[serde(flatten)]
    #[serde(rename(serialize = "version"))]
    pub tag: Tag,
    pub pull_request: Option<PullRequestConfig>,
    pub targets: Targets,
    pub path: Option<String>,
}

impl Brew {
    pub fn new(brew: BrewConfig, version: Tag, packages: Vec<Package>) -> Brew {
        Brew {
            name: captalize(brew.name),
            description: brew.description,
            homepage: brew.homepage,
            install_info: brew.install,
            repository: brew.repository,
            tag: version,
            targets: Targets::from(packages),
            license: brew.license,
            head: brew.head,
            test: brew.test,
            caveats: brew.caveats,
            commit_message: brew.commit_message,
            commit_author: brew.commit_author,
            pull_request: brew.pull_request,
            path: brew.path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewArch {
    pub arch: Arch,
    pub url: String,
    pub hash: String,
}

pub async fn release(
    brew_config: BrewConfig,
    packages: Vec<Package>,
    template: Template,
    base: PathBuf,
    dry_run: bool,
    output_path: &PathBuf,
) -> Result<String> {
    let brew = Brew::new(brew_config, git::get_current_tag(&base)?, packages);

    log::debug!("Rendering Formula template {}", template.to_string());
    let data = serialize_brew(&brew, template)?;

    write_file(output_path.join(format!("{}.rb", brew.name)), &data)?;

    if !dry_run {
        if brew.pull_request.is_some() {
            log::debug!("Creating pull request");
            push_formula(brew).await?;
        } else {
            log::debug!("Committing file to head branch");
            github_client::instance()
                .repo(&brew.repository.owner, &brew.repository.name)
                .branch(&brew.head)
                .upsert_file()
                .path(if brew.path.is_some() {
                    format!("{}/{}.rb", brew.path.unwrap(), brew.name)
                } else {
                    format!("{}.rb", brew.name)
                })
                .message(brew.commit_message.replace("{{version}}", &brew.tag.name))
                .content(&data)
                .execute()
                .await
                .context("error uploading file to main branch")?;
        }
    } else {
        log::debug!("Dry run, not pushing to github or creating pull request");
    }

    Ok(data)
}

fn serialize_brew<T>(data: &T, template: Template) -> Result<String>
where
    T: Serialize,
{
    let hb = handlebars()?;
    let rendered = hb.render(&template.to_string(), data)?;
    Ok(rendered)
}

fn write_file<S>(path: PathBuf, data: S) -> Result<()>
where
    S: Into<String>,
{
    fs::write(path, data.into())?;
    Ok(())
}

fn captalize(mut string: String) -> String {
    format!("{}{string}", string.remove(0).to_uppercase())
}

async fn push_formula(brew: Brew) -> Result<()> {
    let pull_request = brew.pull_request.unwrap();

    let committer: Committer = brew.commit_author.map(Committer::from).unwrap_or_default();

    let repo_handler =
        github_client::instance().repo(&brew.repository.owner, &brew.repository.name);

    log::debug!("Creating branch");
    let sha = repo_handler
        .branch(&pull_request.base)
        .get_commit_sha()
        .await
        .context("error getting the base branch commit sha")?;

    repo_handler
        .branches()
        .create()
        .branch(&pull_request.head)
        .sha(sha.sha)
        .execute()
        .await
        .context("error creating the branch")?;

    let content = fs::read_to_string(format!("{}.rb", brew.name))?;

    log::debug!("Updating formula");
    repo_handler
        .branch(&pull_request.head)
        .upsert_file()
        .path(format!("{}.rb", brew.name))
        .message(brew.commit_message.replace("{{version}}", &brew.tag.name))
        .content(content)
        .committer(&committer)
        .execute()
        .await
        .context("error uploading file to head branch")?;

    log::debug!("Creating pull request");
    repo_handler
        .pull_request()
        .create()
        .assignees(pull_request.assignees.unwrap_or_default())
        .base(pull_request.base)
        .head(&pull_request.head)
        .body(pull_request.body.unwrap_or_default())
        .labels(pull_request.labels.unwrap_or_default())
        .title(pull_request.title.unwrap_or_default())
        .committer(&committer)
        .execute()
        .await
        .context("error creating pull request")?;

    Ok(())
}

impl From<Vec<Package>> for Targets {
    fn from(value: Vec<Package>) -> Targets {
        let v: Vec<Target> = if value.is_empty() {
            vec![]
        } else if value[0].arch.is_none() && value[0].os.is_none() {
            let target = vec![Target::Single(SingleTarget {
                url: value[0].url.clone().unwrap_or_default(),
                hash: value[0].sha256.clone(),
            })];
            target
        } else {
            let group = value
                .iter()
                .cloned()
                .group_by(|p| p.os.to_owned())
                .into_iter()
                .map(|g| MultiTarget {
                    os: g.0.unwrap(),
                    archs: g
                        .1
                        .map(|p| BrewArch {
                            arch: p.arch.to_owned().unwrap(),
                            url: p.url.clone().unwrap_or_default(),
                            hash: p.sha256.clone(),
                        })
                        .collect(),
                })
                .map(Target::Multi)
                .collect();

            group
        };

        Targets(v)
    }
}

impl From<CommitterConfig> for Committer {
    fn from(value: CommitterConfig) -> Self {
        Committer {
            author: value.name,
            email: value.email,
        }
    }
}
