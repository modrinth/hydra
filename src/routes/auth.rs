//! Main authentication flow for Hydra
use crate::stages;

use std::{collections::HashMap, sync::Arc};
use trillium::{conn_try, Conn, Status};
use trillium_askama::Template;
use trillium_client as c;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "socket_response.json")]
struct SockResponse<'a> {
    bearer_token: &'a str,
}

macro_rules! ws_conn_try {
    ($status:path, $res:expr => $conn:expr, $ws_conn:expr) => {
        match $res {
            Ok(res) => res,
            Err(err) => {
                let err = super::socket::render_error(&err.to_string());
                $ws_conn.send_string(err.clone()).await;
                trillium::log_error!($ws_conn.close().await);
                return $conn.with_status($status).with_body(err);
            }
        }
    };
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
        Some(id) => id,
        None => return conn.with_status(Status::BadRequest).with_body(
            "Invalid state sent, you probably need to get a new connection id",
        ),
    };
    let ws_conn = ws_conn.value_mut();

    let host = ws_conn_try!(
        Status::InternalServerError,
        conn.inner().host().ok_or(eyre::eyre!(
            "Tried to use authentication route when own hostname is unknown!"
        )) => conn, ws_conn
    );

    // TODO: integrate sock
    log::info!("Signing in with code {code}");
    let access_token = ws_conn_try!(
        Status::InternalServerError,
        stages::access_token::fetch_token(&client, host, code).await
        => conn, ws_conn
    );

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = ws_conn_try!(
        Status::InternalServerError,
        stages::xbl_signin::login_xbl(&client, &access_token).await
        => conn, ws_conn
    );

    let xsts_response = ws_conn_try!(
        Status::InternalServerError,
        stages::xsts_token::fetch_token(&client, &xbl_token).await
        => conn, ws_conn
    );

    match xsts_response {
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
