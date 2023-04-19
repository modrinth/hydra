//! Get access token from code
use crate::stages::REQWEST_CLIENT;
use serde::Deserialize;
use std::collections::HashMap;

const OAUTH_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
pub const ROUTE_NAME: &str = "auth-redirect";

#[derive(Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn fetch_token(
    public_uri: String,
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> eyre::Result<Tokens> {
    let redirect_uri = format!("{}/{}", public_uri, ROUTE_NAME);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("code", code);
    params.insert("grant_type", "authorization_code");
    params.insert("redirect_uri", redirect_uri.as_str());

    let result = REQWEST_CLIENT
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<Tokens>()
        .await?;

    Ok(result)
}

pub async fn refresh_token(
    public_uri: &str,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> eyre::Result<Tokens> {
    let redirect_uri = format!("{}/{}", public_uri, ROUTE_NAME);

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("refresh_token", refresh_token);
    params.insert("grant_type", "refresh_token");
    params.insert("redirect_uri", &redirect_uri);

    let result = REQWEST_CLIENT
        .post(OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<Tokens>()
        .await?;

    Ok(result)
}
