use anyhow::Result;
use simple_logger::init_with_env;

pub fn init() -> Result<()> {
    init_with_env()?;
    Ok(())
}
