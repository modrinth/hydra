//! Hydra MSA helper
#![deny(unsafe_code, clippy::pedantic)]
#![warn(clippy::nursery)]

mod config;
mod db;
mod pages;
mod routes;
mod stages;

pub(crate) use async_global_executor as executor;
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

fn create_handler(config: Arc<config::Config>) -> impl trillium::Handler {
    (
        State::new(config),
        State::new(Arc::new(db::RuntimeState::default())),
        trillium_head::Head::new(),
        routes::router(),
        trillium_static_compiled::static_compiled!("assets/"),
    )
}

fn main() -> eyre::Result<()> {
    init_logger();
    color_eyre::install()?;

    let config = Arc::new(config::Config::init());
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

// This test won't work in CI, so it has to be manually enabled. `cargo make test` will run it as well
#[cfg(feature = "integration-test")]
#[cfg(test)]
mod test {
    use super::*;
    use eyre::WrapErr;
    use smol::{net::TcpStream, prelude::*};

    #[test]
    fn test_auth() -> eyre::Result<()> {
        init_logger();
        let config = config::Config::init();
        let url = config.public_url.clone();

        let server = async_global_executor::spawn(
            trillium_smol::config()
                .with_port(config.port)
                .with_host(&config.host)
                .run_async(create_handler(Arc::new(config))),
        );

        trillium_testing::block_on(async {
            log::warn!("This integration test requires user interaction. Log in using the opened page to continue.");

            // Open socket
            let sock_url = {
                let mut url = url.clone();
                url.set_scheme("ws").unwrap();
                url
            };

            let sock_addr =
                format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());
            let ws_conn = TcpStream::connect(sock_addr).await.wrap_err(
                "While opening the testing server websocket over TCP",
            )?;
            let (mut sock, _) =
                async_tungstenite::client_async(sock_url.as_str(), ws_conn)
                    .await
                    .wrap_err("While attempting to connect to the websocket")?;

            // Get access code
            let code = match sock.next().await {
                Some(Ok(trillium_websockets::Message::Text(json))) => {
                    let data =
                        serde_json::from_str::<serde_json::Value>(&json)?;

                    if let Some(err) = data.get("error") {
                        eyre::bail!("Error getting auth token: {err}")
                    }

                    let code_raw = data
                        .get("login_code")
                        .and_then(|it| it.as_str().map(String::from))
                        .ok_or(eyre::eyre!(
                            "Successful response contained no login code!"
                        ))?;
                    uuid::Uuid::parse_str(&code_raw)?
                }
                // `rustfmt` seems very confused here
                Some(Err(error)) => {
                    eyre::bail!("Error receiving code over socket: {error}")
                }
                Some(Ok(rsp)) => eyre::bail!(
                    "Received incorrect response type from response: {rsp}",
                ),

                None => {
                    eyre::bail!(
                        "Socket closed before initial response was received!"
                    )
                }
            };

            // Run login flow
            let browser_url = {
                let mut url = url.clone();
                url.set_path("/login");
                url.set_query(Some(&format!("id={code}")));
                url
            };
            webbrowser::open(browser_url.as_str())?;

            // Validate response
            let token = match sock.next().await {
                Some(Ok(trillium_websockets::Message::Text(json))) => {
                    let data =
                        serde_json::from_str::<serde_json::Value>(&json)?;

                    if let Some(err) = data.get("error") {
                        eyre::bail!("Error getting bearer token: {err}")
                    }

                    data.get("token")
                        .and_then(|it| it.as_str().map(String::from))
                        .ok_or(eyre::eyre!(
                            "Successful response contained no bearer token!"
                        ))?
                }
                Some(Err(error)) => {
                    eyre::bail!("Error receiving token over socket: {error}")
                }
                Some(Ok(rsp)) => {
                    eyre::bail!(
                        "Received incorrect response type from response: {rsp}",
                    )
                }
                None => {
                    eyre::bail!(
                        "Socket closed before final response was received!"
                    )
                }
            };

            log::info!("Successfully fetched bearer token: {token}");
            // Allow page to finish loading
            smol::Timer::after(std::time::Duration::from_secs(5)).await;
            server.cancel().await;
            Ok(())
        })
    }
}
