//! Routes for Hydra
use crate::stages;
use trillium::{conn_unwrap, KnownHeaderName, Status};
use trillium_askama::AskamaConnExt;
use trillium_router::{Router, RouterConnExt};

mod auth;
mod login;
mod socket;

pub fn router() -> Router {
    trillium_router::router()
        .get("/teapot", |conn: trillium::Conn| async move {
            conn.render(crate::pages::teapot::Page)
                .with_status(Status::ImATeapot)
        })
        .all("/services/*", |conn: trillium::Conn| async move {
            let route = conn_unwrap!(conn.wildcard().map(String::from), conn);
            conn.with_status(Status::Found).with_header(
                KnownHeaderName::Location,
                format!("https://api.minecraftservices.net/{route}"),
            )
        })
        .get("/login", login::route)
        .get(stages::access_token::ROUTE_NAME, auth::route)
        .get("/", socket::route())
}

#[cfg(test)]
mod test {
    use super::*;
    use trillium_askama::Template;
    use trillium_testing::prelude::*;

    #[test]
    fn teapot() {
        assert_response!(
            get("/teapot").on(&router()),
            Status::ImATeapot,
            crate::pages::teapot::Page.render().unwrap()
        );
    }

    #[test]
    fn minecraft_services() {
        assert_response!(
            get("/services/entitlements/mcstore").on(&router()),
            Status::Found,
            "",
            "Location" => "https://api.minecraftservices.net/entitlements/mcstore"
        );
    }
}
