//! Hydra machine-facing messages
use trillium_askama::Template;

/// Error message
#[derive(Template)]
#[template(path = "messages/error.json")]
#[moretypes::record]
pub struct Error<'a> {
    reason: &'a str,
}

impl<'a> Error<'a> {
    pub fn render(reason: &'a str) -> String {
        Self { reason }.render().unwrap()
    }
}

/// Token fetched successfully
#[derive(Template)]
#[template(path = "messages/bearer.json")]
#[moretypes::record]
pub struct BearerToken<'a> {
    bearer_token: &'a str,
    refresh_token: &'a str,
}

/// Rate limit code acquired
#[derive(Template)]
#[template(path = "messages/ratelimit_code.json")]
#[moretypes::record]
pub struct RateLimitCode<'a> {
    login_code: &'a str,
}
