//! Hydra MSA main binary

mod db;
mod routes;
mod stages;
mod templates;

use actix_files::Files;
use log::{error, info, warn};
use pretty_env_logger::env_logger;
use pretty_env_logger::env_logger::Env;
use std::str::FromStr;

use crate::templates::pages;
use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use tokio::sync::RwLock;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    color_eyre::install().unwrap();

    if check_env_vars() {
        error!("Some environment variables are missing!");
    }

    info!("Starting Hydra on {}", dotenvy::var("BIND_ADDR").unwrap());

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(RwLock::new(db::RuntimeState::default())))
            .app_data(web::FormConfig::default().error_handler(|err, _req| {
                pages::Error {
                    code: StatusCode::BAD_REQUEST,
                    message: err.to_string(),
                }
                .into()
            }))
            .app_data(web::PathConfig::default().error_handler(|err, _req| {
                pages::Error {
                    code: StatusCode::BAD_REQUEST,
                    message: err.to_string(),
                }
                .into()
            }))
            .app_data(web::QueryConfig::default().error_handler(|err, _req| {
                pages::Error {
                    code: StatusCode::BAD_REQUEST,
                    message: err.to_string(),
                }
                .into()
            }))
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                pages::Error {
                    code: StatusCode::BAD_REQUEST,
                    message: err.to_string(),
                }
                .into()
            }))
            .configure(routes::config)
            .service(Files::new("/", "assets/"))
    })
    .workers(1)
    .bind(dotenvy::var("BIND_ADDR").unwrap())?
    .run()
    .await
}

pub fn parse_var<T: FromStr>(var: &'static str) -> Option<T> {
    dotenvy::var(var).ok().and_then(|i| i.parse().ok())
}

// This is so that env vars not used immediately don't panic at runtime
fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: FromStr>(var: &'static str) -> bool {
        let check = parse_var::<T>(var).is_none();
        if check {
            warn!(
                "Variable `{}` missing in dotenv or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
        }
        check
    }

    failed |= check_var::<String>("BIND_ADDR");
    failed |= check_var::<String>("HYDRA_CLIENT_ID");
    failed |= check_var::<String>("HYDRA_CLIENT_SECRET");
    failed |= check_var::<String>("HYDRA_RATE_LIMIT");
    failed |= check_var::<String>("HYDRA_RATE_LIMIT_EXPIRES");

    failed
}
