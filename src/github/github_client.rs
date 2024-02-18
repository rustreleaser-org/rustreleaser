use crate::http;

use super::{asset::Asset, handler::repository_handler::RepositoryHandler, inner::Inner};
use anyhow::Result;
use octocrab::Octocrab;
use once_cell::sync::Lazy;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    multipart::{Form, Part},
};
use std::env;

pub static GITHUB_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"));

static CLIENT: Lazy<GithubClient<Octocrab>> = Lazy::new(GithubClient::load);

pub fn instance() -> &'static GithubClient<Octocrab> {
    &CLIENT
}

/**
 *
 * Github client wrapper
 *
 */
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
    pub fn repos<S>(&self, owner: S, name: S) -> RepositoryHandler
    where
        S: Into<String>,
    {
        RepositoryHandler::new(owner, name)
    }

    pub(super) fn get_inner(&self) -> impl Inner {
        self.inner.to_owned()
    }

    pub(super) async fn upload_asset<S>(
        &self,
        asset: Asset,
        owner: S,
        repo: S,
        release_id: u64,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let client = http::HttpClient::new();
        let content: Vec<u8> = tokio::fs::read(asset.path).await?;
        let part = Part::bytes(content).file_name("file_name.extension");
        let form = Form::new().part("data-binary", part);

        let gh_uri = format!(
            "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name={}",
            owner.into(),
            repo.into(),
            release_id,
            asset.name
        );

        let response = client
            .post(gh_uri)
            .bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(CONTENT_TYPE, "application/octet-stream")
            .multipart(form)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to upload asset"))
        }
    }
}
