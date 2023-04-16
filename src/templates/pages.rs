//! User-facing webpages
use std::fmt::{Debug};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse};
use askama::Template;

/// Successful response
#[derive(Template)]
#[template(path = "success.html")]
pub struct Success<'a> {
    pub uuid: &'a str,
    pub name: &'a str,
}

impl<'a> Success<'a> {
    pub fn render(self) -> HttpResponse {
        HttpResponse::Ok()
            .append_header(("Content-Type", "text/html; charset=utf-8"))
            .body(self.to_string())
    }
}

/// Error response
#[derive(Template, Debug)]
#[template(path = "error.html")]
pub struct Error {
    pub code: StatusCode,
    pub message: String,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.code)
            .append_header(("Content-Type", "text/html; charset=utf-8"))
            .body(self.to_string())
    }
}