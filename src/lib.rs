//! Hydra MSA helper
#![deny(unsafe_code, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

use std::sync::Arc;
use trillium::State;

pub mod config;
pub mod db;
mod routes;
mod stages;
mod templates;

pub type Connector =
    trillium_rustls::RustlsConnector<trillium_smol::TcpConnector>;

pub fn init_logger() {
    let mut builder = pretty_env_logger::formatted_builder();
    builder
        .filter_level(log::LevelFilter::Info)
        .filter_module("trillium_server_common", log::LevelFilter::Warn);

    // HACK: Replicate the .parse_env function, since pretty_env_logger doesn't have a version of env_logger with it implemented
    if let Ok(env) = std::env::var("RUST_LOG") {
        builder.parse_filters(&env);
    }
    builder.init();
}

#[must_use]
pub fn create_handler(config: Arc<config::Config>) -> impl trillium::Handler {
    (
        State::new(config),
        State::new(Arc::new(db::RuntimeState::default())),
        trillium_head::Head::new(),
        routes::router(),
        trillium_static_compiled::static_compiled!("assets/"),
    )
}
