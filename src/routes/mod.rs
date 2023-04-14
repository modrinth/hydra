//! Routes for Hydra
use crate::{stages, templates::pages};
use actix_web::web;

// mod auth;
mod login;
mod refresh;
// mod socket;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login::route).service(refresh::route);
}

// pub fn router() -> Router {
//     trillium_router::router()
//         .get("/login", login::route)
//         .get(stages::access_token::ROUTE_NAME, auth::route)
//         .post("/refresh", refresh::route)
//         .get("/", socket::route())
// }
