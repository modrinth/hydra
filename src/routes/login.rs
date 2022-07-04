//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use crate::config;
use trillium::{Conn, HeaderValue, KnownHeaderName};
use url::Url;

#[allow(clippy::unused_async)]
pub async fn route(conn: Conn) -> Conn {
    log::info!(
        "Redirecting UA {} to Microsoft login page.",
        conn.headers()
            .get(KnownHeaderName::UserAgent)
            .and_then(HeaderValue::as_str)
            .map_or_else(|| String::from("???"), String::from)
    );

    let redirect = conn.inner().host();
    if redirect.is_none() {
        return conn
            .with_status(400)
            .with_body("Tried to use the redirect route without knowing the hostname")
            .halt();
    }

    match Url::parse_with_params(
        "https://login.live.com/oauth20_authorize.srf",
        &[
            ("client_id", config::CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", redirect.unwrap()),
            ("scope", "XboxLive.signin%20offline_access"),
            ("state", "UNNEEDED"),
        ],
    ) {
        Ok(url) => conn
            .with_status(303)
            .with_header(KnownHeaderName::Location, String::from(url.as_str())),
        Err(err) => conn
            .with_status(500)
            .with_body(format!("Error creating redirect URL: {err}"))
            .halt(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use trillium_testing::prelude::*;

    #[test]
    fn test_no_host() {
        assert_response!(get("/").on(&route), Status::BadRequest);
    }
}
