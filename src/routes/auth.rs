//! Main authentication flow for &Hydra
use crate::stages;

use std::collections::HashMap;
use trillium::{conn_try, Conn};
use trillium_client as c;

pub async fn route(conn: Conn) -> Conn {
    let params = url::form_urlencoded::parse(conn.querystring().as_bytes())
        .collect::<HashMap<_, _>>();
    let client = c::Client::<crate::Connector>::new().with_default_pool();
    let code = conn_try!(
        params
            .get("code")
            .ok_or(eyre::eyre!("No access code received")),
        conn
    );
    let host = conn.inner().host();
    if host.is_none() {
        return conn
            .with_status(400)
            .with_body(
                "Tried to use the redirect route without knowing the hostname",
            )
            .halt();
    }

    log::info!("Signing in with code {}", code);
    let access_token = conn_try!(
        stages::access_token::fetch_access_token(&client, host.unwrap(), code)
            .await,
        conn
    );
    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = conn_try!(
        stages::xbl_signin::login_xbl(&client, &access_token).await,
        conn
    );

    log::debug!("Fetched token: {xbl_token} (hash: {uhs})");
    conn
}
