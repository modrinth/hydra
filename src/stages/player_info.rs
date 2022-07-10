//! Fetch player info for display
use serde::Deserialize;
use trillium::{KnownHeaderName, Status};
use trillium_client as c;

const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Deserialize)]
#[moretypes::record]
pub struct PlayerInfo {
    name: String,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            name: String::from("???"),
        }
    }
}

pub async fn fetch_info(
    client: &c::Client<crate::Connector>,
    token: &str,
) -> Option<PlayerInfo> {
    log::info!("GET {PROFILE_URL}");
    let mut resp = client
        .get(PROFILE_URL)
        .with_header(KnownHeaderName::Authorization, format!("Bearer {token}"));
    resp.send().await.ok()?;
    resp.status().filter(|it| *it == Status::Ok)?;

    serde_json::from_slice::<PlayerInfo>(
        &resp.response_body().read_bytes().await.ok()?,
    )
    .ok()
}
