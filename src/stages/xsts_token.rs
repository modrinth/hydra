//! XSTS token fetcher
use trillium::{KnownHeaderName, Status};
use trillium_askama::Template;
use trillium_client as c;

const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

#[derive(Template)]
#[template(path = "xsts_body.json", escape = "none")]
struct XSTSBodyTemplate<'a> {
    user_token: &'a str,
}

pub enum XSTSResponse {
    Unauthorized(String),
    Success { token: String },
}

pub async fn fetch_token(
    client: &c::Client<crate::Connector>,
    token: &str,
) -> eyre::Result<XSTSResponse> {
    let body = XSTSBodyTemplate { user_token: token }.render()?;

    log::trace!("POST {XSTS_AUTH_URL}");
    let mut req = client
        .post(XSTS_AUTH_URL)
        .with_header(KnownHeaderName::ContentType, "application/json")
        .with_header(KnownHeaderName::Accept, "application/json")
        .with_body(body);
    req.send().await?;

    let body = req.response_body().read_string().await?;
    log::trace!("Received response: {body}");

    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    match req.status().unwrap() {
        Status::Ok => Some(json)
            .and_then(|it| it.get("Token")?.as_str().map(String::from))
            .map(|it| XSTSResponse::Success { token: it })
            .ok_or(eyre::eyre!("XSTS response didn't contain valid token!")),
        Status::Forbidden => Ok(XSTSResponse::Unauthorized(
            #[allow(clippy::unreadable_literal)]
            match json.get("XErr").unwrap().as_i64().unwrap() {
                2148916238 => String::from(
                    "Underage XBox Live account needs to be added to a family",
                ),
                2148916233 => {
                    String::from("Could not find valid XBox live account!")
                }
                _ => unreachable!(),
            },
        )),
        _ => unreachable!(),
    }
}
