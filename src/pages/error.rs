//! Error page
use trillium_askama::{AskamaConnExt, Template};

#[derive(Template)]
#[template(path = "pages/error.html")]
pub struct Page<'a> {
    pub code: &'a trillium::Status,
    pub message: &'a str,
}

impl<'a> Page<'a> {
    pub fn render(self, conn: trillium::Conn) -> trillium::Conn {
        let status = *self.code;
        conn.render(self).with_status(status).halt()
    }
}
