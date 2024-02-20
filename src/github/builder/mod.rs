pub mod create_branch_builder;
pub mod create_pull_request_builder;
pub mod create_release_builder;
pub mod upsert_file_builder;

use anyhow::Result;

pub trait BuilderExecutor {
    type Output;

    async fn execute(self) -> Result<Self::Output>;
}
