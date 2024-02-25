use crate::github::release::{Release, ReleaseResponse};

use super::{
    asset::{Asset, UploadedAsset},
    builder::create_pull_request_builder::Committer,
    handler::repository_handler::RepositoryHandler,
    model::{pull_request::PullRequest, sha::Sha},
};
use anyhow::{bail, Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use once_cell::sync::Lazy;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, USER_AGENT},
    multipart::{Form, Part},
};
use serde_json::json;
use std::{collections::HashMap, env};

#[macro_export]
macro_rules! get {
    ($url:expr) => {
        $crate::http::HttpClient::new()
            .get($url)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .send()
            .await?
    };
}

#[macro_export]
macro_rules! post {
    ($url:expr, $body:expr) => {
        $crate::http::HttpClient::new()
            .post($url)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .body($body)
            .send()
            .await?
    };
}

#[macro_export]
macro_rules! form {
    ($url:expr, $form:expr) => {
        $crate::http::HttpClient::new()
            .post($url)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .header(CONTENT_TYPE, "application/octet-stream")
            .multipart($form)
            .send()
            .await
    };
}

#[macro_export]
macro_rules! put {
    ($url:expr, $body:expr) => {
        $crate::http::HttpClient::new()
            .put($url)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rust-releaser")
            .body($body)
            .send()
            .await?
    };
}

pub static GITHUB_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"));

static CLIENT: Lazy<GithubClient> = Lazy::new(|| GithubClient);

pub fn instance() -> &'static GithubClient {
    &CLIENT
}

pub struct GithubClient;

impl GithubClient {
    pub fn repo<S>(&self, owner: S, name: S) -> RepositoryHandler
    where
        S: Into<String>,
    {
        RepositoryHandler::new(owner, name)
    }

    pub(super) async fn upload_asset(
        &self,
        asset: &Asset,
        owner: impl Into<String>,
        tag: impl Into<String>,
        repo: impl Into<String>,
        release_id: u64,
    ) -> Result<UploadedAsset> {
        let content: Vec<u8> = tokio::fs::read(&asset.path).await?;

        let part = Part::bytes(content).file_name(asset.name.to_owned());

        let form = Form::new().part("data-binary", part);

        let owner = owner.into();
        let repo = repo.into();

        let uri = format!(
            "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name={}",
            &owner, &repo, release_id, asset.name
        );

        let response = form!(uri, form);

        match response {
            Ok(response) => {
                log::debug!("status: {}", response.status());
                let response = response.text().await?;
                log::debug!("response: {}", response);
            }
            Err(err) => {
                bail!(err);
            }
        }

        let tag: String = tag.into();
        let tag = if tag.starts_with('v') {
            tag.strip_prefix('v').unwrap_or_default()
        } else {
            &tag
        };
        let asset_url = format!(
            "https://github.com/{}/{}/releases/download/v{}/{}",
            &owner, &repo, &tag, asset.name
        );
        log::info!("creating uploaded asset");
        let uploaded_asset = self.create_uploaded_asset(asset, asset_url);

        Ok(uploaded_asset)
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

        let response = get!(&uri);

        let response = response.text().await?;

        let sha = Sha { sha: response };

        Ok(sha)
    }

    pub async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        sha: &str,
    ) -> Result<()> {
        let uri = format!("https://api.github.com/repos/{}/{}/git/refs", owner, repo);

        let mut body = HashMap::new();

        body.insert("ref", format!("refs/heads/{}", branch));
        body.insert("sha", sha.to_owned());

        let body: String = serde_json::to_string(&body)?;

        post!(&uri, body);

        Ok(())
    }

    pub async fn update_file(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        content: &str,
        commit_message: String,
        committer: Committer,
        head: String,
    ) -> Result<()> {
        let content = BASE64_STANDARD.encode(content.as_bytes());

        let mut committer_map = HashMap::new();

        committer_map.insert("name", committer.author);
        committer_map.insert("email", committer.email);

        let committer = serde_json::to_string(&committer_map)?;

        let file_sha = get!(&format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        ))
        .text()
        .await
        .context("failed to get Formula sha value")?;

        let sha = serde_json::from_str::<Sha>(&file_sha).unwrap_or_default();

        let body = if sha.sha.is_empty() {
            let mut body = HashMap::new();

            body.insert("message", commit_message);
            body.insert("branch", head);
            body.insert("content", content);
            body.insert("committer", committer);
            body.insert("sha", sha.sha);

            let body: String = serde_json::to_string(&body)?;

            body
        } else {
            let mut body = HashMap::new();

            body.insert("message", commit_message);
            body.insert("content", content);
            body.insert("committer", committer);
            body.insert("sha", sha.sha);

            let body: String = serde_json::to_string(&body)?;

            body
        };

        let uri = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        put!(uri, body).text().await?;

        Ok(())
    }

    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        pr_body: Option<String>,
        assigness: Vec<String>,
        labels: Vec<String>,
    ) -> Result<PullRequest> {
        let uri = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);
        let pr_body = pr_body.unwrap_or("".to_string());
        let mut body = HashMap::new();

        body.insert("title", title);
        body.insert("head", head);
        body.insert("base", base);
        body.insert("body", &pr_body);

        let body: String = serde_json::to_string(&body)?;

        let response = post!(&uri, body).text().await?;

        let pr: PullRequest = serde_json::from_str(&response)?;

        if !assigness.is_empty() {
            self.set_pr_assignees(owner, repo, pr.number, assigness)
                .await?;
        }

        if !labels.is_empty() {
            self.set_pr_labels(owner, repo, pr.number.to_string(), labels)
                .await?;
        }

        Ok(pr)
    }

    async fn set_pr_assignees(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        assigness: Vec<String>,
    ) -> Result<()> {
        let uri = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/assignees",
            owner, repo, pr_number
        );

        let mut body = HashMap::new();

        body.insert("assignees", assigness);

        let body: String = serde_json::to_string(&body)?;

        post!(&uri, body).text().await?;

        Ok(())
    }

    async fn set_pr_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: String,
        labels: Vec<String>,
    ) -> Result<()> {
        let uri = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/labels",
            owner, repo, pr_number
        );

        let mut body = HashMap::new();

        body.insert("labels", labels);

        let body: String = serde_json::to_string(&body)?;

        post!(&uri, body).text().await?;

        Ok(())
    }

    pub async fn create_release(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        target_branch: &str,
        release_name: &str,
        draft: bool,
        prerelease: bool,
    ) -> Result<Release> {
        let uri = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);

        let body = json!({
            "tag_name": tag,
            "target_commitish": target_branch,
            "name": release_name,
            "draft": draft,
            "prerelease": prerelease
        });

        let body: String = serde_json::to_string(&body)?;

        let response = post!(&uri, body).text().await?;

        let release = serde_json::from_str::<ReleaseResponse>(&response)?;
        Ok(Release::new(release.id, owner, repo))
    }

    pub async fn get_release_by_tag(&self, owner: &str, repo: &str, tag: &str) -> Result<Release> {
        let uri = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}",
            owner, repo, tag
        );

        let response = get!(&uri).text().await?;
        let release = serde_json::from_str::<ReleaseResponse>(&response)?;

        Ok(Release::new(release.id, owner, repo))
    }
}
