//! User-facing webpages
use actix_web::http::StatusCode;
use askama::Template;

/// Successful response
#[derive(Template)]
#[template(path = "success.html")]
pub struct Success<'a> {
    pub name: &'a str,
}

/// Error response
#[derive(Template)]
#[template(path = "error.html")]
pub struct Error<'a> {
    pub code: StatusCode,
    pub message: &'a str,
}

// impl<'a> Error<'a> {
//     pub fn render(self, conn: trillium::Conn) -> trillium::Conn {
//         let status = self.code;
//         conn.render(self).with_status(status).halt()
//     }
// }
