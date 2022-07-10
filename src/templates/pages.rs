//! User-facing webpages
use trillium_askama::{AskamaConnExt, Template};

/// Successful response
#[derive(Template)]
#[template(path = "pages/success.html")]
pub struct Success<'a> {
    pub name: &'a str,
}

/// Error response
#[derive(Template)]
#[template(path = "pages/error.html")]
pub struct Error<'a> {
    pub code: trillium::Status,
    pub message: &'a str,
}

impl<'a> Error<'a> {
    pub fn render(self, conn: trillium::Conn) -> trillium::Conn {
        let status = self.code;
        conn.render(self).with_status(status).halt()
    }
}

/// I'm a teapot!
#[derive(Template)]
#[template(path = "pages/teapot.html")]
pub struct Teapot;
