//! I'm a teapot!
use trillium_askama::Template;

#[derive(Template)]
#[template(path = "pages/teapot.html")]
pub struct Page;
