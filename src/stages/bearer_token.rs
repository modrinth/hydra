//! Minecraft bearer token
use trillium::KnownHeaderName;
use trillium_askama::Template;
use trillium_client as c;

const MCSERVICES_AUTH_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";

#[derive(Template)]
#[template(path = "bearer_body.json", escape = "none")]
struct BearerBodyTemplate<'a> {
    user_hash: &'a str,
    xsts_token: &'a str,
}

pub async fn fetch_bearer(
    client: &c::Client<crate::Connector>,
    token: &str,
    uhs: &str,
) -> eyre::Result<String> {
    let body = BearerBodyTemplate {
        user_hash: uhs,
        xsts_token: token,
    }
    .render()?;

    log::trace!("POST {MCSERVICES_AUTH_URL}");
    let mut req = client
        .post(MCSERVICES_AUTH_URL)
        .with_header(KnownHeaderName::ContentType, "application/json")
        .with_body(body);
    req.send().await?;

    let body = req.response_body().read_string().await?;
    log::trace!("Received response: {body}");

    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    json.get("access_token")
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .ok_or(eyre::eyre!(
            "Minecraft Services API didn't contain valid bearer token"
        ))
}
