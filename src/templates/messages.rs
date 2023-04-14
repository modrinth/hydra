//! Hydra machine-facing messages
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde_json::json;

/// Error message
pub struct Error<'a> {
    pub code: StatusCode,
    pub reason: &'a str,
}

impl<'a> Error<'a> {
    pub fn render(self) -> HttpResponse {
        HttpResponse::build(self.code).json(json!({
            "error": self.reason
        }))
    }
}

/// Rate limit code acquired
pub struct RateLimitCode<'a> {
    pub login_code: &'a str,
}
