//! Minecraft bearer token
use crate::stages::REQWEST_CLIENT;
use serde_json::json;

const MCSERVICES_AUTH_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";

pub async fn fetch_bearer(token: &str, uhs: &str) -> eyre::Result<String> {
    let body = REQWEST_CLIENT
        .post(MCSERVICES_AUTH_URL)
        .json(&json!({
            "identityToken": format!("XBL3.0 x={};{}", token, uhs),
            "ensureLegacyEnabled": true
        }))
        .send()
        .await?
        .text()
        .await?;

    serde_json::from_str::<serde_json::Value>(&body)?
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .ok_or(eyre::eyre!("Response didn't contain valid bearer token"))
}
