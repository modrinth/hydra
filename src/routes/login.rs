//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use crate::{parse_var, stages::login_redirect, templates::pages};
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Query {
    pub id: Option<String>,
}

#[derive(Serialize)]
pub struct AuthorizationInit {
    pub url: String,
}

#[get("login")]
#[allow(clippy::unused_async)]
pub async fn route(info: web::Query<Query>) -> Result<HttpResponse, pages::Error> {
    let conn_id = info.0.id.ok_or_else(|| pages::Error {
        code: StatusCode::BAD_REQUEST,
        message: "No socket ID provided (open a web socket at the / route for one)".to_string(),
    })?;

    let public_url = parse_var::<String>("HYDRA_PUBLIC_URL").unwrap_or(format!(
        "http://{}",
        parse_var::<String>("BIND_ADDR").unwrap()
    ));
    let client_id = parse_var::<String>("HYDRA_CLIENT_ID").unwrap();

    let url = login_redirect::get_url(&public_url, &conn_id, &client_id).map_err(|err| pages::Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Error creating login URL: {err}"),
    })?;

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", &*url))
        .json(AuthorizationInit { url }))
}
