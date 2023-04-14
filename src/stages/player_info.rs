//! Fetch player info for display
use crate::stages::REQWEST_CLIENT;
use serde::Deserialize;

const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Deserialize)]
pub struct PlayerInfo {
    pub name: String,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            name: String::from("???"),
        }
    }
}

pub async fn fetch_info(token: &str) -> eyre::Result<PlayerInfo> {
    let resp = REQWEST_CLIENT
        .get(PROFILE_URL)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp)
}
