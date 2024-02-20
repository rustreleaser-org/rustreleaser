use super::{
    builder::{
        create_pull_request_builder::CreatePullRequestBuilder,
        create_release_builder::CreateReleaseBuilder,
    },
    pull_request::PullRequest,
    release::Release,
};
use anyhow::Result;

pub trait Inner: Clone {
    async fn get_release_by_tag<'tag>(
        &self,
        owner: String,
        repo: String,
        tag: &'tag str,
    ) -> Result<Release>;

    async fn create_release(&self, builder: CreateReleaseBuilder) -> Result<Release>;

    async fn create_pull_request(&self, builder: CreatePullRequestBuilder) -> Result<PullRequest>;
}
