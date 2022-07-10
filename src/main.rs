//! Hydra MSA main binary

use async_global_executor as executor;
use hydra::*;
use std::sync::Arc;

#[cfg(feature = "tls")]
use trillium_rustls::RustlsAcceptor;

fn main() -> eyre::Result<()> {
    init_logger();
    color_eyre::install()?;

    let config = Arc::new(config::Config::init()?);
    log::info!("Starting Hydra at {}:{}", config.host, config.port);

    #[cfg(feature = "tls")]
    let get_cert = executor::spawn({
        let config = Arc::clone(&config);

        async move {
            use smol::{fs, future};
            log::debug!("Reading TLS certificates");

            future::try_zip(fs::read(&config.cert_file), fs::read(&config.key_file))
                .await
                .map_err(|err| eyre::eyre!(
                    "Error loading TLS certificates: {err}. You may need to create them or disable the tls feature."
                ))
        }
    });

    log::debug!("Loading config");
    let cfg = trillium_smol::config()
        .with_port(config.port)
        .with_host(&config.host);

    #[cfg(feature = "tls")]
    let cfg = executor::block_on(async {
        log::debug!("Giving certificates to Rustls");
        let (tls_cert, tls_key) = get_cert.await?;
        Ok::<_, eyre::Error>(
            cfg.with_acceptor(RustlsAcceptor::from_pkcs8(&tls_cert, &tls_key)),
        )
    })?;

    executor::block_on(async {
        let server = cfg.run_async(create_handler(Arc::clone(&config)));
        log::info!("Started Hydra!");
        server.await;
        Ok(())
    })
}
