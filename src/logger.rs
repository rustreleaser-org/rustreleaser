use anyhow::Result;
use simple_logger::init_with_level;

pub fn init() -> Result<()> {
    init_with_level(log::Level::Debug)?;
    Ok(())
}
