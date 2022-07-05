//! Main authentication flow for Hydra
use crate::stages;

use std::collections::HashMap;
use trillium::{conn_try, Conn, Status};
use trillium_askama::{AskamaConnExt, Template};
use trillium_client as c;

#[derive(Template)]
#[template(path = "auth_response.json", escape = "none")]
struct RouteResponse<'a> {
    bearer_token: &'a str,
}

// TODO: Rate limit?
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
            .with_status(Status::BadRequest)
            .with_body(
                "Tried to use the redirect route without knowing the hostname",
            )
            .halt();
    }

    log::info!("Signing in with code {code}");
    let access_token = conn_try!(
        stages::access_token::fetch_token(&client, host.unwrap(), code).await,
        conn
    );

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = conn_try!(
        stages::xbl_signin::login_xbl(&client, &access_token).await,
        conn
    );

    match conn_try!(
        stages::xsts_token::fetch_token(&client, &xbl_token).await,
        conn
    ) {
        stages::xsts_token::XSTSResponse::Unauthorized(err) => {
            conn.with_status(Status::Forbidden).with_body(err).halt()
        }
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = conn_try!(
                stages::bearer_token::fetch_bearer(&client, &xsts_token, &uhs)
                    .await,
                conn
            );
            log::info!("Signin for code {code} successful");
            conn.render(RouteResponse {
                bearer_token: &bearer_token,
            })
        }
    }
}
