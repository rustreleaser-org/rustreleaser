use anyhow::Result;
use handlebars::{handlebars_helper, Handlebars};

pub fn handlebars<'hb>() -> Result<Handlebars<'hb>> {
    let mut hb = Handlebars::new();

    let template = include_str!("./multi_target.hbs");

    hb.register_template_string("multi_target", template)?;

    handlebars_helper!(eq: |this: str, other: str| this.eq(other));

    hb.register_helper("eq", Box::new(eq));

    Ok(hb)
}
