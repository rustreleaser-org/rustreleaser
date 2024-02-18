use crate::github::{
    builder::create_release_builder::CreateReleaseBuilder,
    github_client::{GithubClient, GITHUB_TOKEN},
    inner::Inner,
    release::Release,
};
use anyhow::Result;
use octocrab::Octocrab;

impl GithubClient<Octocrab> {
    pub fn load() -> GithubClient<Octocrab> {
        let github_token: &str = &GITHUB_TOKEN;

        let inner_handler = if let Ok(octo) = Octocrab::builder()
            .personal_token(github_token.to_owned())
            .build()
        {
            octo
        } else {
            panic!("Failed to create client's inner handler");
        };

        GithubClient {
            inner: inner_handler,
        }
    }
}

impl Inner for Octocrab {
    async fn get_release_by_tag<'tag>(
        &self,
        owner: String,
        repo: String,
        tag: &'tag str,
    ) -> Result<Release> {
        let release = self
            .repos(owner.to_owned(), repo.to_owned())
            .releases()
            .get_by_tag(tag)
            .await?;

        Ok(Release::new(release.id.0, owner, repo))
    }

    async fn create_release(&self, builder: CreateReleaseBuilder) -> Result<Release> {
        let r = self
            .repos(&builder.owner, &builder.repo)
            .releases()
            .create(&builder.release_tag.expect("Release tag must be set"))
            .target_commitish(&builder.target_branch.expect("Target branch must be set"))
            .draft(builder.draft.expect("Draft must be set"))
            .name(&builder.release_name.expect("Release name must be set"))
            .prerelease(builder.prerelease.expect("Pre-release must be set"))
            .send()
            .await?;

        Ok(Release::new(r.id.0, builder.owner, builder.repo))
    }
}
