//! Routes for Hydra
use trillium_router::Router;

mod auth;
mod login;

pub use trillium_askama::AskamaConnExt;
pub use trillium_router::RouterConnExt;

pub fn router() -> Router {
    trillium_router::routes!(
        get "/teapot" |conn: trillium::Conn| async move {
            conn.with_status(418).with_body("I'm short and stout!")
        },
        get "/login" login::route,
        get "/auth-redirect" auth::route,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use trillium_testing::prelude::*;

    #[test]
    fn simple() {
        assert_response!(
            get("/teapot").on(&router()),
            Status::ImATeapot,
            "I'm short and stout!"
        )
    }
}
