pub mod release_handler;
pub mod releases_handler;
pub mod repository_handler;

use anyhow::Result;

pub trait HandlerExecutor {
    type Output;
    async fn execute(self) -> Result<Self::Output>;
}
