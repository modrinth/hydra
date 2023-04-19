//! Main authentication flow for Hydra
use crate::{
    parse_var, stages,
    templates::{messages, pages},
};

use crate::db::RuntimeState;
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

macro_rules! ws_conn_try {
    ($ctx:literal $status:path, $res:expr => $ws_conn:expr) => {
        match $res {
            Ok(res) => res,
            Err(err) => {
                let error = format!("In {}: {err}", $ctx);
                let render = messages::Error::render_string(&error);
                let _ = $ws_conn.text(render.clone()).await;
                let _ = $ws_conn.close(None).await;
                return Err(pages::Error {
                    code: $status,
                    message: render,
                });
            }
        }
    };
}

#[derive(Deserialize)]
pub struct Query {
    pub code: String,
    pub state: String,
}

#[get("auth-redirect")]
pub async fn route(
    db: web::Data<RuntimeState>,
    info: web::Query<Query>,
) -> Result<HttpResponse, pages::Error> {
    let public_url = parse_var::<String>("HYDRA_PUBLIC_URL").unwrap_or(format!(
        "http://{}",
        parse_var::<String>("BIND_ADDR").unwrap()
    ));
    let client_id = parse_var::<String>("HYDRA_CLIENT_ID").unwrap();
    let client_secret = parse_var::<String>("HYDRA_CLIENT_SECRET").unwrap();

    let code = &info.code;

    let mut ws_conn = Uuid::try_parse(&info.state)
        .ok()
        .and_then(|it| db.auth_sockets.get_mut(&it))
        .ok_or_else(|| pages::Error {
            code: StatusCode::BAD_REQUEST,
            message: "Invalid state sent, you probably need to get a new websocket".to_string(),
        })?;
    let mut ws_conn = ws_conn.value_mut().clone();

    let access_token = ws_conn_try!(
        "OAuth token exchange" StatusCode::INTERNAL_SERVER_ERROR,
        stages::access_token::fetch_token(
            public_url,
            code,
            &client_id,
            &client_secret,
        ).await
        => ws_conn
    );

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = ws_conn_try!(
        "XBox Live token exchange" StatusCode::INTERNAL_SERVER_ERROR,
        stages::xbl_signin::login_xbl(&access_token.access_token).await
        => ws_conn
    );

    let xsts_response = ws_conn_try!(
        "XSTS token exchange" StatusCode::INTERNAL_SERVER_ERROR,
        stages::xsts_token::fetch_token(&xbl_token).await
        => ws_conn
    );

    match xsts_response {
        stages::xsts_token::XSTSResponse::Unauthorized(err) => {
            let _ = ws_conn
                .text(messages::Error::render_string(&format!(
                    "Error getting XBox Live token: {err}"
                )))
                .await;
            let _ = ws_conn.close(None).await;

            Err(pages::Error {
                code: StatusCode::FORBIDDEN,
                message: err,
            })
        }
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = &ws_conn_try!(
                "Bearer token flow" StatusCode::INTERNAL_SERVER_ERROR,
                stages::bearer_token::fetch_bearer(&xsts_token, &uhs)
                    .await
                => ws_conn
            );

            ws_conn
                .text(
                    json!({
                        "token": bearer_token,
                        "refresh_token": &access_token.refresh_token,
                        "expires_after": 86400
                    }).to_string()
                )
                .await.map_err(|_| pages::Error {
                code: StatusCode::BAD_REQUEST,
                message: "Failed to send login details to launcher. Try restarting the login process!".to_string(),
            })?;
            let _ = ws_conn.close(None).await;

            let player_info = stages::player_info::fetch_info(bearer_token)
                .await
                .unwrap_or_default();

            Ok(pages::Success {
                name: &player_info.name,
                uuid: &player_info.id,
            }
            .render())
        }
    }
}
