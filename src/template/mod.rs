use crate::build::Build;
use anyhow::Result;
use handlebars::{handlebars_helper, Handlebars};

pub fn handlebars<'hb>() -> Result<Handlebars<'hb>> {
    let mut hb = Handlebars::new();

    let multi_target = include_str!("./multi_target.hbs");
    let single_target = include_str!("./single_target.hbs");

    hb.register_template_string("multi_target", multi_target)?;
    hb.register_template_string("single_target", single_target)?;

    handlebars_helper!(eq: |this: str, other: str| this.eq(other));

    hb.register_helper("eq", Box::new(eq));

    Ok(hb)
}

pub enum Template {
    MultiTarget,
    SingleTarget,
}

impl ToString for Template {
    fn to_string(&self) -> String {
        match self {
            Template::MultiTarget => "multi_target".to_string(),
            Template::SingleTarget => "single_target".to_string(),
        }
    }
}

impl From<Build> for Template {
    fn from(build: Build) -> Self {
        match build.is_multi_target() {
            true => Template::MultiTarget,
            false => Template::SingleTarget,
        }
    }
}
