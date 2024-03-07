use super::{
    asset::{Asset, UploadedAsset},
    handler::repository_handler::RepositoryHandler,
    request::{
        branch_ref_request::BranchRefRequest, create_release_request::CreateReleaseRequest,
        pull_request_request::PullRquestRequest,
    },
    response::{
        assignees_request::AssigneesRequest, labels_request::LabelsRequest,
        pull_request_response::PullRequest, release_response::ReleaseResponse, sha_response::Sha,
    },
    tag::Tag,
};
use crate::{
    build::committer::Committer,
    get,
    github::{macros::Headers, release::Release, request::upsert_file_request::UpsertFileRequest},
    http::HttpClient,
    post, put,
};
use anyhow::{Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use log::debug;
use mime_guess::from_path;
use once_cell::sync::Lazy;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use std::{env, path::Path};
use tokio::{fs::File, io::AsyncReadExt};

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
        tag: &Tag,
        repo: impl Into<String>,
        release_id: u64,
    ) -> Result<UploadedAsset> {
        let owner = owner.into();
        let repo = repo.into();

        let path = Path::new(&asset.path);
        let mut file = File::open(&path).await?;
        let metadata = file.metadata().await?;
        let content_length = metadata.len();
        let content_type = from_path(&path)
            .first_or_octet_stream()
            .as_ref()
            .to_string();

        let url = format!(
            "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name={}",
            owner, repo, release_id, asset.name
        );

        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).await?;
        let res = HttpClient::new()
            .post(url)
            .default_headers()
            .header(CONTENT_LENGTH, content_length.to_string())
            .header(CONTENT_TYPE, content_type)
            .body(buf)
            .send()
            .await?;

        debug!("upload asset response: {:#?}", res);

        let asset_url = format!(
            "https://github.com/{}/{}/releases/download/{}/{}",
            &owner,
            &repo,
            tag.strip_v_prefix(),
            asset.name
        );
        log::debug!("creating uploaded asset");
        let uploaded_asset = self.create_uploaded_asset(asset, asset_url);

        Ok(uploaded_asset)
    }

    pub(super) fn create_uploaded_asset(&self, asset: &Asset, url: String) -> UploadedAsset {
        UploadedAsset::new(
            asset.name.to_owned(),
            url,
            asset
                .checksum
                .as_ref()
                .unwrap_or(&String::default())
                .to_owned(),
        )
    }

    pub(super) async fn get_commit_sha(
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

        let response = get!(&uri)?;

        let sha = Sha { sha: response };

        Ok(sha)
    }

    pub(super) async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        sha: &str,
    ) -> Result<()> {
        let uri = format!("https://api.github.com/repos/{}/{}/git/refs", owner, repo);

        let request = BranchRefRequest::new(branch.to_string(), sha.to_string());

        let body: String = serde_json::to_string(&request)?;

        post!(&uri, body)?;

        Ok(())
    }

    pub(super) async fn upsert_file(
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

        let uri = &format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        let file_sha = get!(uri).context("failed to get Formula sha value")?;

        let sha = serde_json::from_str::<Sha>(&file_sha).unwrap_or_default();

        let body = if sha.sha.is_empty() {
            log::debug!("creating new file");

            let request =
                UpsertFileRequest::new(commit_message, content, Some(head), None, committer.into());

            serde_json::to_string(&request)?
        } else {
            log::debug!("updating file");

            let request = UpsertFileRequest::new(
                commit_message,
                content,
                Some(head),
                Some(sha.sha),
                committer.into(),
            );

            serde_json::to_string(&request)?
        };

        let uri = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        put!(uri, body)?;

        Ok(())
    }

    pub(super) async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        pr_body: &str,
        assignees: Vec<String>,
        labels: Vec<String>,
    ) -> Result<PullRequest> {
        let uri = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

        let request = PullRquestRequest::new(
            title.to_owned(),
            head.to_owned(),
            base.to_owned(),
            pr_body.to_owned(),
        );
        let body: String = serde_json::to_string(&request)?;

        let response = post!(&uri, body)?;

        let pr: PullRequest = serde_json::from_str(&response)?;

        if !assignees.is_empty() {
            self.set_pr_assignees(owner, repo, pr.number, assignees)
                .await?;
        }

        if !labels.is_empty() {
            self.set_pr_labels(owner, repo, pr.number.to_string(), labels)
                .await?;
        }

        Ok(pr)
    }

    pub(super) async fn create_release(
        &self,
        owner: &str,
        repo: &str,
        tag: &Tag,
        target_branch: &str,
        release_name: &str,
        draft: bool,
        prerelease: bool,
        body: &str,
    ) -> Result<Release> {
        let uri = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);

        let request = CreateReleaseRequest::new(
            tag.value().to_owned(),
            target_branch.to_owned(),
            release_name.to_owned(),
            body.to_owned(),
            draft,
            prerelease,
        );

        let body: String = serde_json::to_string(&request)?;

        let response = post!(&uri, body)?;

        let release = serde_json::from_str::<ReleaseResponse>(&response)?;

        Ok(Release::new(release.id, owner, repo))
    }

    pub(super) async fn get_release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &Tag,
    ) -> Result<Release> {
        let uri = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}",
            owner,
            repo,
            tag.value()
        );

        let response = get!(&uri)?;

        let release = serde_json::from_str::<ReleaseResponse>(&response)?;
        debug!("release: {:#?}", release);
        Ok(Release::new(release.id, owner, repo))
    }

    async fn set_pr_assignees(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        assignees: Vec<String>,
    ) -> Result<()> {
        let uri = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/assignees",
            owner, repo, pr_number
        );

        let request = AssigneesRequest::new(assignees);

        let body: String = serde_json::to_string(&request)?;

        post!(&uri, body)?;

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

        let request = LabelsRequest::new(labels);

        let body: String = serde_json::to_string(&request)?;

        post!(&uri, body)?;

        Ok(())
    }
}
