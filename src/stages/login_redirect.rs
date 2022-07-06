//! Login redirect step
use crate::config;
use trillium_askama::Template;

#[derive(Template)]
#[template(path = "authorize_url")]
struct LoginTemplate<'a> {
    client_id: &'a str,
    redirect_uri: &'a str,
    conn_id: &'a str,
}

pub fn get_url(host: &str, conn_id: &str) -> eyre::Result<String> {
    let data = LoginTemplate {
        client_id: config::CLIENT_ID,
        redirect_uri: &super::get_redirect_url(host),
        conn_id,
    };
    Ok(data.render()?)
}
