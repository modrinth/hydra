//! Login redirect step
use trillium_askama::Template;

#[derive(Template)]
#[template(path = "authorize_url")]
struct LoginTemplate<'a> {
    client_id: &'a str,
    redirect_uri: &'a str,
    conn_id: &'a str,
}

pub fn get_url(
    public_uri: &url::Url,
    conn_id: &str,
    client_id: &str,
) -> eyre::Result<String> {
    LoginTemplate {
        client_id,
        redirect_uri: public_uri
            .join(super::access_token::ROUTE_NAME)?
            .as_str(),
        conn_id,
    }
    .render()
    .map_err(eyre::Error::from)
}
