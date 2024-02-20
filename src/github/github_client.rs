use super::{
    asset::{Asset, UploadedAsset},
    builder::create_pull_request_builder::Commiter,
    handler::repository_handler::RepositoryHandler,
    inner::Inner,
    sha::Sha,
};
use crate::http;
use anyhow::{bail, Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use octocrab::Octocrab;
use once_cell::sync::Lazy;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, USER_AGENT},
    multipart::{Form, Part},
};
use std::{collections::HashMap, env};

pub static GITHUB_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"));

static CLIENT: Lazy<GithubClient<Octocrab>> = Lazy::new(GithubClient::load);

pub fn instance() -> &'static GithubClient<Octocrab> {
    &CLIENT
}

pub struct GithubClient<I>
where
    I: Inner + Clone,
{
    pub(super) inner: I,
}

impl<I> GithubClient<I>
where
    I: Inner,
{
    pub fn repo<S>(&self, owner: S, name: S) -> RepositoryHandler
    where
        S: Into<String>,
    {
        RepositoryHandler::new(owner, name)
    }

    pub(super) fn get_inner(&self) -> impl Inner {
        self.inner.to_owned()
    }

    pub(super) async fn upload_asset(
        &self,
        asset: &Asset,
        owner: impl Into<String>,
        tag: impl Into<String>,
        repo: impl Into<String>,
        release_id: u64,
    ) -> Result<UploadedAsset> {
        let client = http::HttpClient::new();
        let content: Vec<u8> = tokio::fs::read(&asset.path).await?;
        let part = Part::bytes(content).file_name(asset.name.to_owned());
        let form = Form::new().part("data-binary", part);

        let owner = owner.into();
        let repo = repo.into();

        let uri = format!(
            "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name={}",
            &owner, &repo, release_id, asset.name
        );

        let response = client
            .post(&uri)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(CONTENT_TYPE, "application/octet-stream")
            .multipart(form)
            .send()
            .await?;

        if response.status().is_success() {
            let tag = tag.into();
            let current_tag = if tag.is_empty() { "9.9.9" } else { &tag };
            let asset_url = format!(
                "https://github.com/{}/{}/releases/download/v{}/{}",
                &owner, &repo, current_tag, asset.name
            );

            log::debug!("uploading asset {:#?}", asset);
            let uploaded_asset = self.create_uploaded_asset(asset, asset_url);

            Ok(uploaded_asset)
        } else {
            let msg = response.text().await?;
            log::error!("Failed to upload asset: {:#?}", msg);
            bail!(anyhow::anyhow!("Failed to upload asset"))
        }
    }

    pub fn create_uploaded_asset(&self, asset: &Asset, url: String) -> UploadedAsset {
        UploadedAsset::new(
            asset.name.to_owned(),
            url,
            asset
                .checksum
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_owned(),
        )
    }

    pub async fn get_commit_sha(
        &self,
        owner: impl Into<String>,
        repo: impl Into<String>,
        base: impl Into<String>,
    ) -> Result<Sha> {
        let owner = owner.into();
        let repo = repo.into();
        let base = base.into();

        let uri = format!(
            "https://api.github.com/repos/{}/{}/commits/{}",
            &owner, &repo, &base
        );

        let client = http::HttpClient::new();
        let response = client
            .get(&uri)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .send()
            .await?;

        let sha = Sha {
            sha: response.text().await?,
        };

        Ok(sha)
    }

    pub async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        sha: &str,
    ) -> Result<()> {
        let client = http::HttpClient::new();
        let uri = format!("https://api.github.com/repos/{}/{}/git/refs", owner, repo);

        let mut body = HashMap::new();

        body.insert("ref", format!("refs/heads/{}", branch));
        body.insert("sha", sha.to_owned());

        let body: String = serde_json::to_string(&body)?;

        let response = client
            .post(uri)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .body(body)
            .send()
            .await?;

        println!("response: {}", response.text().await?);

        Ok(())
    }

    pub async fn update_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        commit_message: Option<String>,
        commiter: Commiter,
        head: String,
    ) -> Result<()> {
        let client = http::HttpClient::new();

        let file_sha = client
            .get(&format!(
                "https://api.github.com/repos/{}/{}/contents/{}",
                owner, repo, path
            ))
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .send()
            .await?
            .text()
            .await
            .context("failed to get Formula sha value")?;

        let sha = serde_json::from_str::<Sha>(&file_sha).unwrap_or(Sha {
            sha: "".to_string(),
        });

        let message = commit_message.unwrap_or("update formula".to_string());
        let content = BASE64_STANDARD.encode(content.as_bytes());

        let mut commiter_map = HashMap::new();

        commiter_map.insert("name", commiter.author);
        commiter_map.insert("email", commiter.email);

        let commiter = serde_json::to_string(&commiter_map)?;

        let body = if sha.sha.is_empty() {
            log::info!("creating file");

            let mut body = HashMap::new();

            body.insert("message", message);
            body.insert("branch", head);
            body.insert("content", content);
            body.insert("commiter", commiter);
            body.insert("sha", sha.sha);

            let body: String = serde_json::to_string(&body)?;

            body
        } else {
            log::info!("updating file");

            let mut body = HashMap::new();

            body.insert("message", message);
            body.insert("content", content);
            body.insert("commiter", commiter);
            body.insert("sha", sha.sha);

            let body: String = serde_json::to_string(&body)?;

            body
        };

        let uri = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        let response = client
            .put(uri)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .body(body)
            .send()
            .await?
            .text()
            .await?;

        log::info!("response: {}", response);

        Ok(())
    }
}
