//! Login redirect step

use askama::filters::urlencode;

pub fn get_url(public_uri: &url::Url, conn_id: &str, client_id: &str) -> eyre::Result<String> {
    Ok(
        format!(
            "https://login.live.com/oauth20_authorize.srf?client_id={client_id}&response_type=code&redirect_uri={}&scope={}&state={conn_id}",
            urlencode(public_uri.join(super::access_token::ROUTE_NAME)?.as_str())?,
            urlencode("XboxLive.signin offline_access")?
        )
    )
}
