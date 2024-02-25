pub mod install;
pub mod package;
pub mod repository;

use self::{install::Install, package::Package, repository::Repository};
use crate::{
    build::{arch::Arch, os::Os},
    config::{BrewConfig, CommitterConfig, PullRequestConfig},
    git,
    github::{
        builder::{create_pull_request_builder::Commiter, BuilderExecutor},
        github_client,
    },
    template::{handlebars, Template},
};
use anyhow::{Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brew {
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub head: Option<String>,
    pub test: Option<String>,
    pub caveats: Option<String>,
    pub commit_message: Option<String>,
    pub commit_author: Option<CommitterConfig>,
    pub install_info: Install,
    pub repository: Repository,
    pub version: String,
    pub pull_request: Option<PullRequestConfig>,
    pub targets: Targets,
}

impl Brew {
    pub fn new(brew: BrewConfig, version: String, packages: Vec<Package>) -> Brew {
        Brew {
            name: captalize(brew.name),
            description: brew.description,
            homepage: brew.homepage,
            install_info: brew.install,
            repository: brew.repository,
            version,
            targets: Targets::from(packages),
            license: brew.license,
            head: brew.head,
            test: brew.test,
            caveats: brew.caveats,
            commit_message: brew.commit_message,
            commit_author: brew.commit_author,
            pull_request: brew.pull_request,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Targets(pub Vec<Target>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTarget {
    pub os: Os,
    pub archs: Vec<BrewArch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleTarget {
    pub url: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Single(SingleTarget),
    Multi(MultiTarget),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewArch {
    pub arch: Arch,
    pub url: String,
    pub hash: String,
}

impl From<Vec<Package>> for Targets {
    fn from(value: Vec<Package>) -> Targets {
        let v: Vec<Target> = if value.len() == 1 {
            let target = vec![Target::Single(SingleTarget {
                url: value[0].url.clone(),
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
                            url: p.url.clone(),
                            hash: p.sha256.clone(),
                        })
                        .collect(),
                })
                .map(|t| Target::Multi(t))
                .collect();

            group
        };

        Targets(v)
    }
}

pub async fn release(
    brew_config: BrewConfig,
    packages: Vec<Package>,
    is_multitarget: bool,
) -> Result<String> {
    log::info!("Creating brew file");

    let brew = Brew::new(brew_config, git::get_current_tag()?, packages);
    log::info!("Rendering Formula template");
    let template = if is_multitarget {
        Template::MultiTarget
    } else {
        Template::SingleTarget
    };

    let data = serialize_brew(&brew, template)?;

    write_file(format!("{}.rb", brew.name), &data)?;

    if brew.pull_request.is_some() {
        log::info!("Creating pull request");
        push_formula(brew).await?;
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

fn write_file<S>(file_name: String, data: S) -> Result<()>
where
    S: Into<String>,
{
    fs::write(file_name, data.into())?;
    Ok(())
}

fn captalize(mut s: String) -> String {
    format!("{}{s}", s.remove(0).to_uppercase())
}

async fn push_formula(brew: Brew) -> Result<()> {
    let pull_request = brew.pull_request.unwrap();

    let commiter: Commiter = brew.commit_author.map(|c| c.into()).unwrap_or_default();

    let head_branch = pull_request
        .head
        .unwrap_or("bumps-formula-version".to_owned());

    let base_branch = pull_request.base.unwrap_or("main".to_owned());

    let repo_handler =
        github_client::instance().repo(&brew.repository.owner, &brew.repository.name);

    let sha = repo_handler
        .branch(&base_branch)
        .get_commit_sha()
        .await
        .context("error getting the base branch commit sha")?;

    repo_handler
        .branches()
        .create()
        .branch(&head_branch)
        .sha(sha.sha)
        .execute()
        .await
        .context("error creating the branch")?;

    let content = fs::read_to_string(format!("{}.rb", brew.name))?;

    repo_handler
        .branch(&head_branch)
        .upsert_file()
        .path(format!("{}.rb", brew.name))
        .message(brew.commit_message.unwrap_or("update formula".to_owned()))
        .content(content)
        .commiter(&commiter)
        .execute()
        .await
        .context("error uploading file to head branch")?;

    repo_handler
        .pull_request()
        .create()
        .assignees(pull_request.assignees.unwrap_or_default())
        .base(base_branch)
        .head(head_branch)
        .body(pull_request.body.unwrap_or_default())
        .labels(pull_request.labels.unwrap_or_default())
        .title(pull_request.title.unwrap_or_default())
        .commiter(&commiter)
        .execute()
        .await
        .context("error creating pull request")?;

    Ok(())
}

impl From<CommitterConfig> for Commiter {
    fn from(value: CommitterConfig) -> Self {
        Commiter {
            author: value.name,
            email: value.email,
        }
    }
}
