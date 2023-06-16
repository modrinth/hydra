//! XSTS token fetcher

use crate::stages::REQWEST_CLIENT;
use serde_json::json;

const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

pub enum XSTSResponse {
    Unauthorized(String),
    Success { token: String },
}

pub async fn fetch_token(token: &str) -> eyre::Result<XSTSResponse> {
    let resp = REQWEST_CLIENT
        .post(XSTS_AUTH_URL)
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    token
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?;
    let status = resp.status();

    let body = resp.text().await?;
    let json = serde_json::from_str::<serde_json::Value>(&body)?;

    if status.is_success() {
        Some(json)
            .and_then(|it| it.get("Token")?.as_str().map(String::from))
            .map(|it| XSTSResponse::Success { token: it })
            .ok_or(eyre::eyre!("XSTS response didn't contain valid token!"))
    } else {
        Ok(XSTSResponse::Unauthorized(
            #[allow(clippy::unreadable_literal)]
            match json.get("XErr").and_then(|x| x.as_i64()) {
                Some(2148916238) => {
                    String::from("This Microsoft account is underage and is not linked to a family.")
                },
                Some(2148916235) => {
                    String::from("XBOX Live/Minecraft is not available in your country.")
                },
                Some(2148916233) => String::from("This account does not have a valid XBOX Live profile. Please buy Minecraft and try again!"),
                Some(2148916236) | Some(2148916237) => String::from("This account needs adult verification on Xbox page."),
                _ => String::from("Unknown error code"),
            },
        ))
    }
}
