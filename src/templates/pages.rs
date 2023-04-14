//! User-facing webpages
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder};
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

impl<'a> Error<'a> {
    pub fn render(self) -> HttpResponse {
        let status = self.code;

        HttpResponse::build(status)
            .append_header(("Content-Type", "text/html; charset=utf-8"))
            .body(self.to_string())
    }
}
