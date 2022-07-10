//! Routes for Hydra
use crate::{stages, templates::pages};
use trillium::Status;
use trillium_askama::AskamaConnExt;
use trillium_router::Router;

mod auth;
mod login;
mod refresh;
mod socket;

pub fn router() -> Router {
    trillium_router::router()
        .get("/teapot", |conn: trillium::Conn| async move {
            conn.render(pages::Teapot).with_status(Status::ImATeapot)
        })
        .get("/login", login::route)
        .get(stages::access_token::ROUTE_NAME, auth::route)
        .post("/refresh", refresh::route)
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
            pages::Teapot.render().unwrap()
        );
    }
}
