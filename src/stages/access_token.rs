//! Get access token from code
use serde::Deserialize;
use trillium::KnownHeaderName;
use trillium_askama::Template;
use trillium_client as c;

const OAUTH_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
pub const ROUTE_NAME: &str = "/auth-redirect";

#[derive(Template)]
#[template(path = "bodies/oauth_token")]
struct Body<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    auth_code: &'a str,
    redirect_uri: &'a str,
}

#[derive(Template)]
#[template(path = "bodies/refresh_token")]
struct RefreshBody<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    refresh_token: &'a str,
    redirect_uri: &'a str,
}

#[derive(Deserialize)]
#[moretypes::record]
pub struct Tokens {
    access_token: String,
    refresh_token: String,
}

pub async fn fetch_token(
    client: &c::Client<crate::Connector>,
    public_uri: &url::Url,
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> eyre::Result<Tokens> {
    let body = Body {
        client_id,
        client_secret,
        auth_code: code,
        redirect_uri: public_uri.join(ROUTE_NAME)?.as_str(),
    }
    .render()?;

    log::info!("POST {OAUTH_TOKEN_URL}");
    let mut req = client
        .post(OAUTH_TOKEN_URL)
        .with_header(
            KnownHeaderName::ContentType,
            "application/x-www-form-urlencoded",
        )
        .with_body(body);
    req.send().await?;

    let body = req.response_body().read_string().await?;
    log::trace!("Received response: {body}");

    Ok(serde_json::from_str::<Tokens>(&body)?)
}

pub async fn refresh_token(
    client: &c::Client<crate::Connector>,
    public_uri: &url::Url,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> eyre::Result<Tokens> {
    let body = RefreshBody {
        client_id,
        client_secret,
        refresh_token,
        redirect_uri: public_uri.join(ROUTE_NAME)?.as_str(),
    }
    .render()?;

    log::info!("POST {OAUTH_TOKEN_URL}");
    let mut req = client
        .post(OAUTH_TOKEN_URL)
        .with_header(
            KnownHeaderName::ContentType,
            "application/x-www-form-urlencoded",
        )
        .with_body(body);
    req.send().await?;

    let body = req.response_body().read_string().await?;
    log::trace!("Received response: {body}");

    Ok(serde_json::from_str::<Tokens>(&body)?)
}
