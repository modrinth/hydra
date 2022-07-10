//! Refresh token route
use crate::{stages, templates::messages};
use serde::Deserialize;
use std::sync::Arc;
use trillium::{Conn, KnownHeaderName, Status};
use trillium_askama::AskamaConnExt;
use trillium_client as c;

macro_rules! conn_try {
    ($ctx:literal $status:path, $res:expr => $conn:expr) => {
        match $res {
            Ok(res) => res,
            Err(err) => {
                return $conn
                    .with_body(messages::Error::render(&format!(
                        "In {}: {err} ",
                        $ctx
                    )))
                    .with_header(
                        KnownHeaderName::ContentType,
                        "application/json",
                    )
                    .with_status($status)
                    .halt()
            }
        }
    };
}

#[derive(Deserialize)]
struct Body<'a> {
    refresh_token: &'a str,
}

pub async fn route(mut conn: Conn) -> Conn {
    let client = c::Client::<crate::Connector>::new().with_default_pool();
    let config = conn.state::<Arc<crate::config::Config>>().unwrap().clone();

    let body = conn_try!(
        "Reading the request body" Status::BadRequest,
        conn.request_body().await.read_bytes().await
        => conn
    );
    // TODO: content-type header?
    let body = conn_try!(
        "Parsing the request body" Status::BadRequest,
        serde_json::from_slice::<Body>(&body)
        => conn
    );

    log::info!("Logging in with refresh token {}", body.refresh_token);
    let access_token = conn_try!(
        "OAuth token exchange" Status::InternalServerError,
        stages::access_token::refresh_token(
            &client,
            &config.public_url,
            body.refresh_token,
            &config.client_id,
            &config.client_secret,
        ).await
        => conn
    );

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = conn_try!(
        "XBox Live token exchange" Status::InternalServerError,
        stages::xbl_signin::login_xbl(&client, &access_token.access_token).await
        => conn
    );

    let xsts_response = conn_try!(
        "XSTS token exchange" Status::InternalServerError,
        stages::xsts_token::fetch_token(&client, &xbl_token).await
        => conn
    );

    match xsts_response {
        stages::xsts_token::XSTSResponse::Unauthorized(err) => conn
            .with_body(messages::Error::render(&format!(
                "Error getting XBox Live token: {err}"
            )))
            .with_status(Status::Unauthorized)
            .halt(),
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = &conn_try!(
                "Bearer token flow" Status::InternalServerError,
                stages::bearer_token::fetch_bearer(&client, &xsts_token, &uhs)
                    .await
                => conn
            );
            conn.render(messages::BearerToken {
                bearer_token,
                refresh_token: &access_token.refresh_token,
            })
            .with_header(KnownHeaderName::ContentType, "application/json")
        }
    }
}
