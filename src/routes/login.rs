//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use crate::stages::login_redirect;
use eyre::WrapErr;
use std::collections::HashMap;
use trillium::{conn_try, Conn, HeaderValue, KnownHeaderName, Status};

#[allow(clippy::unused_async)]
pub async fn route(conn: Conn) -> Conn {
    log::info!(
        "Redirecting UA {} to Microsoft login page.",
        conn.headers()
            .get(KnownHeaderName::UserAgent)
            .and_then(HeaderValue::as_str)
            .map_or_else(|| String::from("???"), String::from)
    );

    let query = url::form_urlencoded::parse(conn.querystring().as_bytes())
        .collect::<HashMap<_, _>>();
    let conn_id = match query.get("id") {
        Some(id) => id,
        // TODO: better error page
        None => return conn.with_status(Status::BadRequest).with_body(
            "No socket ID provided (open a web socket at the / route for one)",
        ),
    };

    let host = conn_try!(
        conn.inner().host().ok_or(eyre::eyre!(
            "Server cannot determine hostname for redirect."
        )),
        conn
    );
    let url = conn_try!(
        login_redirect::get_url(host, conn_id)
            .wrap_err("Failed to create login URL"),
        conn
    );

    log::trace!("GET {url}");
    conn.with_status(Status::SeeOther)
        .with_header(KnownHeaderName::Location, url)
}

#[cfg(test)]
mod test {
    use super::*;
    use trillium_testing::prelude::*;

    #[test]
    fn test_no_host() {
        assert_response!(get("/").on(&route), Status::BadRequest);
    }
}
