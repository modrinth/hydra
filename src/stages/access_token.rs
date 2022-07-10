//! Get access token from code
use trillium::KnownHeaderName;
use trillium_askama::Template;
use trillium_client as c;

const OAUTH_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
pub const ROUTE_NAME: &str = "/auth-redirect";

#[derive(Template)]
#[template(path = "bodies/oauth_token")]
struct AccessTokenTemplate<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    auth_code: &'a str,
    redirect_uri: &'a str,
}

pub async fn fetch_token(
    client: &c::Client<crate::Connector>,
    public_uri: &url::Url,
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> eyre::Result<String> {
    let body = AccessTokenTemplate {
        client_id,
        client_secret,
        auth_code: code,
        redirect_uri: public_uri.join(ROUTE_NAME)?.as_str(),
    }
    .render()?;

    log::info!("POST {OAUTH_TOKEN_URL} (code: {code})");
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

    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    json.get("access_token")
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .ok_or(eyre::eyre!("Response didn't contain valid access token"))
}
