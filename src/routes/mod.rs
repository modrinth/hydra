//! Routes for Hydra
use actix_web::web;

mod auth;
mod login;
mod refresh;
mod socket;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login::route).service(refresh::route).service(socket::route).service(auth::route);
}
