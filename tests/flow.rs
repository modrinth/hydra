// These tests won't work in CI, so they have to be manually enabled. `cargo make test` will run it as well
use eyre::{Result, WrapErr};
use hydra::*;
use serde::Deserialize;
use smol::{net::TcpStream, prelude::*};
use std::sync::Arc;
use trillium_client as c;

#[derive(Deserialize)]
struct Token {
    token: String,
    refresh_token: String,
}

async fn test_main_flow(url: &url::Url) -> Result<Token> {
    // Open socket
    let sock_url = {
        let mut url = url.clone();
        url.set_scheme("ws").unwrap();
        url
    };

    let sock_addr =
        format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());
    let ws_conn = TcpStream::connect(sock_addr)
        .await
        .wrap_err("While opening the testing server websocket over TCP")?;
    let (mut sock, _) =
        async_tungstenite::client_async(sock_url.as_str(), ws_conn)
            .await
            .wrap_err("While attempting to connect to the websocket")?;

    // Get access code
    let code = match sock.next().await {
        Some(Ok(trillium_websockets::Message::Text(json))) => {
            let data = serde_json::from_str::<serde_json::Value>(&json)?;

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
        Some(Ok(rsp)) => {
            eyre::bail!("Received incorrect response type from response: {rsp}",)
        }

        None => {
            eyre::bail!("Socket closed before initial response was received!")
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
            let data = serde_json::from_str::<serde_json::Value>(&json)?;

            if let Some(err) = data.get("error") {
                eyre::bail!("Error getting bearer token: {err}")
            }

            serde_json::from_value::<Token>(data)?
        }
        Some(Err(error)) => {
            eyre::bail!("Error receiving token over socket: {error}")
        }
        Some(Ok(rsp)) => {
            eyre::bail!("Received incorrect response type from response: {rsp}",)
        }
        None => {
            eyre::bail!("Socket closed before final response was received!")
        }
    };

    log::info!("Successfully fetched bearer token: {}", &token.token);
    Ok(token)
}

async fn test_refresh_flow(url: &url::Url, tokens: &Token) -> Result<()> {
    // Run refresh flow
    let mut resp = c::Conn::<hydra::Connector>::post(url.join("/refresh")?)
        .with_body(serde_json::to_string_pretty(&serde_json::json!({
            "refresh_token": &tokens.refresh_token
        }))?)
        .execute()
        .await?;

    let new_tokens = serde_json::from_slice::<Token>(
        &resp.response_body().read_bytes().await?,
    )?;

    log::info!(
        "Successfully refreshed token: {} => {}",
        tokens.token,
        new_tokens.token
    );
    Ok(())
}

#[test]
#[ignore]
fn test_auth() -> Result<()> {
    init_logger();
    let config = config::Config::init()?;
    let url = config.public_url.clone();

    let server = async_global_executor::spawn(
        trillium_smol::config()
            .with_port(config.port)
            .with_host(&config.host)
            .run_async(create_handler(Arc::new(config))),
    );

    trillium_testing::block_on(async {
        log::warn!("This integration test requires user interaction. Log in using the opened page to continue.");
        let tokens = test_main_flow(&url).await?;
        // Buffer in some time between requests
        smol::Timer::after(std::time::Duration::from_secs(3)).await;
        test_refresh_flow(&url, &tokens).await?;

        // Allow everything to finish loading
        smol::Timer::after(std::time::Duration::from_secs(3)).await;
        server.cancel().await;
        Ok(())
    })
}
