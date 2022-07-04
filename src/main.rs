//! Hydra MSA helper
#![deny(unsafe_code, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod routes;

use std::sync::Arc;
use trillium::State;

#[cfg(feature = "tls")]
use trillium_rustls::RustlsAcceptor;

fn main() -> eyre::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_module("hydra", log::LevelFilter::Info)
        .init();
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
        Ok::<_, eyre::Error>(cfg.with_acceptor(RustlsAcceptor::from_pkcs8(&tls_cert, &tls_key)))
    }))?;

    log::info!("Started Hydra!");
    smol::block_on(exec.run(async {
        cfg.run_async((
            State::new(db.await?),
            trillium_head::Head::new(),
            routes::router(),
        ))
        .await;
        Ok(())
    }))
}
