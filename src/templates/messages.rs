//! Hydra machine-facing messages
use serde::Serialize;

/// Error message
#[derive(Serialize)]
pub struct Error<'a> {
    pub error: &'a str,
}

impl<'a> Error<'a> {
    pub fn render(reason: &'a str) -> String {
        serde_json::to_string(&Self { error: reason }).unwrap()
    }
}

/// Token fetched successfully
#[derive(Serialize)]
pub struct BearerToken<'a> {
    // bearer token
    pub token: &'a str,
    pub refresh_token: &'a str,
    // always 86400
    pub expires_after: i32,
}

/// Rate limit code acquired
#[derive(Serialize)]
pub struct RateLimitCode<'a> {
    pub login_code: &'a str,
}
