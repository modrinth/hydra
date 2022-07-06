//! Main authentication flow for Hydra
use crate::stages;

use std::{collections::HashMap, sync::Arc};
use trillium::{conn_try, Conn, Status};
use trillium_askama::Template;
use trillium_client as c;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "sock_response.json")]
struct SockResponse<'a> {
    bearer_token: &'a str,
}

pub async fn route(conn: Conn) -> Conn {
    let params = url::form_urlencoded::parse(conn.querystring().as_bytes())
        .collect::<HashMap<_, _>>();
    let client = c::Client::<crate::Connector>::new().with_default_pool();
    let state = conn
        .state::<Arc<crate::db::RuntimeState>>()
        .unwrap()
        .clone();

    let code = conn_try!(
        params
            .get("code")
            .ok_or(eyre::eyre!("No access code received")),
        conn
    );

    let conn_id = match params.get("state") {
        Some(id) => id.clone().into_owned(),
        None => {
            return conn
                .with_status(Status::BadRequest)
                .with_body("No state sent, are you using the redirect?")
        }
    };
    let mut ws_conn = match Uuid::try_parse(&conn_id)
        .ok()
        .and_then(|it| state.auth_sockets.get_mut(&it))
    {
        Some(sock) => sock,
        None => return conn.with_status(Status::BadRequest).with_body(
            "State was not associated with a websocket, are you using the redirect?",
        ),
    };
    let ws_conn = ws_conn.value_mut();

    let host = conn_try!(
        conn.inner().host().ok_or(eyre::eyre!(
            "Tried to use authentication route when own hostname is unknown!"
        )),
        conn
    );

    // TODO: integrate sock
    log::info!("Signing in with code {code}");
    let access_token = conn_try!(
        stages::access_token::fetch_token(&client, host, code).await,
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
            ws_conn
                .send_string(super::socket::render_error(&format!(
                    "Error getting XBox Live token: {err}"
                )))
                .await;
            trillium::log_error!(ws_conn.close().await);
            conn.with_status(Status::Forbidden).with_body(err).halt()
        }
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = &conn_try!(
                stages::bearer_token::fetch_bearer(&client, &xsts_token, &uhs)
                    .await,
                conn
            );
            log::info!("Signin for code {code} successful");
            ws_conn
                .send_string(SockResponse { bearer_token }.render().unwrap())
                .await;
            trillium::log_error!(ws_conn.close().await);

            // TODO: Response page
            conn.ok("Done")
        }
    }
}
