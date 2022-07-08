//! MSA authentication stages

pub mod access_token;
pub mod bearer_token;
pub mod login_redirect;
pub mod xbl_signin;
pub mod xsts_token;

#[inline]
pub(self) fn get_redirect_url(host: &str) -> String {
    #[cfg(not(feature = "tls"))]
    let method = match host.starts_with("localhost") {
        true => "http",
        false => "https",
    };
    #[cfg(feature = "tls")]
    let method = "https";

    format!("{method}://{host}/auth-redirect")
}
