use serde::{Deserialize, Serialize};
use trillium_askama::Template;

#[derive(Template)]
#[template(path = "pages/success.html")]
pub struct Page<'a> {
    pub name: &'a str,
}

#[derive(Serialize, Deserialize)]
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
