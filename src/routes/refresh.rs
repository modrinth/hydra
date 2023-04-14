//! Refresh token route
use crate::templates::pages;
use crate::{parse_var, stages, templates::messages};
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use url::Url;

#[derive(Deserialize)]
pub struct Body {
    refresh_token: String,
}

#[post("refresh")]
pub async fn route(body: web::Json<Body>) -> HttpResponse {
    let public_url = parse_var::<String>("HYDRA_PUBLIC_URL").unwrap_or(format!(
        "http://{}",
        parse_var::<String>("BIND_ADDR").unwrap()
    ));
    let client_id = parse_var::<String>("HYDRA_CLIENT_ID").unwrap();
    let client_secret = parse_var::<String>("HYDRA_CLIENT_SECRET").unwrap();

    let access_token = match stages::access_token::refresh_token(
        &Url::parse(&public_url).unwrap(),
        &body.refresh_token,
        &client_id,
        &client_secret,
    )
    .await {
        Ok(val) => val,
        Err(_) => return messages::Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: &format!("Error with OAuth token exchange"),
        }
            .render()
    };

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = match stages::xbl_signin::login_xbl(&access_token.access_token)
        .await {
        Ok(val) => val,
        Err(_) => return messages::Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: &format!("Error with XBox Live token exchange"),
        }
            .render()
    };

    let xsts_response = match stages::xsts_token::fetch_token(&xbl_token)
        .await {
        Ok(val) => val,
        Err(_) => return messages::Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: &format!("Error with XSTS token exchange"),
        }
            .render()
    };

    match xsts_response {
        stages::xsts_token::XSTSResponse::Unauthorized(err) => messages::Error {
            code: StatusCode::UNAUTHORIZED,
            reason: &format!("Error getting XBox Live token: {err}"),
        }
        .render(),
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = match stages::bearer_token::fetch_bearer(&xsts_token, &uhs)
                .await {
                Ok(val) => val,
                Err(_) => return messages::Error {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    reason: &format!("Error with Bearer token flow"),
                }
                    .render()
            };

            HttpResponse::Ok().json(&json!({
                "token": bearer_token,
                "refresh_token": &access_token.refresh_token,
                "expires_after": 86400
            }))
        }
    }
}
