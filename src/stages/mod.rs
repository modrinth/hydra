//! MSA authentication stages

pub mod access_token;
pub mod login_redirect;
pub mod xbl_signin;
pub mod xsts_token;

#[inline]
pub(self) fn get_redirect_url(host: &str) -> String {
    let prefix = if host.starts_with("localhost") {
        "http"
    } else {
        "https"
    };
    format!("{prefix}://{host}/auth-redirect")
}
