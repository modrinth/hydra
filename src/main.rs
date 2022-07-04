//! Hydra MSA helper
#![deny(unsafe_code, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod routes;
mod stages;

use std::sync::Arc;
use trillium::State;

#[cfg(feature = "tls")]
use trillium_rustls::RustlsAcceptor;

pub type Connector =
    trillium_rustls::RustlsConnector<trillium_smol::TcpConnector>;

fn init_logger() {
    let mut builder = pretty_env_logger::formatted_builder();
    builder
        .filter_level(log::LevelFilter::Info)
        .filter_module("trillium_server_common", log::LevelFilter::Warn)
        .filter_module("sled", log::LevelFilter::Error);

    // HACK: Replicate the .parse_env function, since pretty_env_logger doesn't have a version of env_logger with it implemented
    if let Ok(env) = std::env::var("RUST_LOG") {
        builder.parse_filters(&env);
    }
    builder.init();
}

fn main() -> eyre::Result<()> {
    // HACK: Pretty env logger doesn't support the .parse_env method, so this forces it
    init_logger();
    color_eyre::install()?;

    let config = Arc::new(config::Config::init());
    log::info!("Starting Hydra at {}:{}", config.host, config.port);
    let exec = smol::Executor::new();

    #[cfg(feature = "tls")]
    let get_cert = exec.spawn(async {
        log::debug!("Reading TLS certificates");
        use smol::{fs, future};

        future::try_zip(fs::read(&config.cert_file), fs::read(&config.key_file))
            .await
            .map_err(|err| eyre::eyre!(
                "Error loading TLS certificates: {err}. You may need to create them or disable the tls feature."
            ))
    });

    let db = exec.spawn(async {
        log::debug!("Opening database");
        let config = Arc::clone(&config);
        smol::unblock(move || sled::Config::from(config.as_ref()).open()).await
    });

    log::debug!("Loading config");
    let cfg = trillium_smol::config()
        .with_port(config.port)
        .with_host(&config.host);

    #[cfg(feature = "tls")]
    let cfg = smol::block_on(exec.run(async {
        log::debug!("Giving certificates to Rustls");
        let (tls_cert, tls_key) = get_cert.await?;
        Ok::<_, eyre::Error>(
            cfg.with_acceptor(RustlsAcceptor::from_pkcs8(&tls_cert, &tls_key)),
        )
    }))?;

    smol::block_on(exec.run(async {
        let server = cfg.run_async((
            State::new(db.await?),
            trillium_head::Head::new(),
            routes::router(),
        ));
        log::info!("Started Hydra!");
        server.await;
        Ok(())
    }))
}
