//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use crate::{pages, stages::login_redirect};
use std::collections::HashMap;
use trillium::{Conn, HeaderValue, KnownHeaderName, Status};

#[allow(clippy::unused_async)]
pub async fn route(conn: Conn) -> Conn {
    log::info!(
        "Redirecting UA {} to Microsoft login page.",
        conn.headers()
            .get(KnownHeaderName::UserAgent)
            .and_then(HeaderValue::as_str)
            .map_or_else(|| String::from("???"), String::from)
    );
    let config = conn
        .state::<std::sync::Arc<crate::config::Config>>()
        .unwrap()
        .clone();

    let query = url::form_urlencoded::parse(conn.querystring().as_bytes())
        .collect::<HashMap<_, _>>();
    let conn_id = match query.get("id") {
        Some(id) => id,
        None => return pages::error::Page {
            code: &Status::BadRequest,
            message: "No socket ID provided (open a web socket at the / route for one)"
        }.render(conn)
    };

    let host = match conn.inner().host() {
        Some(host) => host,
        None => {
            return pages::error::Page {
                code: &Status::InternalServerError,
                message: "Server cannot determine the MSA redirect hostname.",
            }
            .render(conn)
        }
    };

    let url = match login_redirect::get_url(host, conn_id, &config.client_id) {
        Ok(url) => url,
        Err(err) => {
            return pages::error::Page {
                code: &Status::InternalServerError,
                message: &format!("Error creating login URL: {err}"),
            }
            .render(conn)
        }
    };

    log::trace!("GET {url}");
    conn.with_status(Status::SeeOther)
        .with_header(KnownHeaderName::Location, url)
}
