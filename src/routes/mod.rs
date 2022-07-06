//! Routes for Hydra
use trillium::Status;
use trillium_router::Router;

mod auth;
mod login;
mod socket;

pub fn router() -> Router {
    trillium_router::routes!(
        get "/teapot" |conn: trillium::Conn| async move {
            conn.with_status(Status::ImATeapot).with_body("I'm short and stout!")
        },
        get "/login" login::route,
        get "/auth-redirect" auth::route,
        get "/" socket::route(),
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
