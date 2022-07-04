//! Signin for XBox Live
use trillium::KnownHeaderName;
use trillium_askama::Template;
use trillium_client as c;

const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";

#[derive(Template)]
#[template(path = "xbl_body.json", escape = "none")]
struct XBLBodyTemplate<'a> {
    access_token: &'a str,
}

pub struct XBLLogin {
    pub token: String,
    pub uhs: String,
}

pub async fn login_xbl(
    client: &c::Client<crate::Connector>,
    token: &str,
) -> eyre::Result<XBLLogin> {
    let body = XBLBodyTemplate {
        access_token: token,
    }
    .render()?;

    log::trace!("POST {XBL_AUTH_URL}");
    let mut req = client
        .post(XBL_AUTH_URL)
        .with_header(KnownHeaderName::ContentType, "application/json")
        .with_header(KnownHeaderName::Accept, "application/json")
        .with_body(body);
    req.send().await?;

    let body = req.response_body().read_string().await?;
    log::trace!("Received response: {body}");

    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    let token = Some(&json)
        .and_then(|it| it.get("Token")?.as_str().map(String::from))
        .ok_or(eyre::eyre!("XBL response didn't contain valid token"))?;
    let uhs = Some(&json)
        .and_then(|it| {
            it.get("DisplayClaims")?
                .get("xui")?
                .get(0)?
                .get("uhs")?
                .as_str()
                .map(String::from)
        })
        .ok_or(eyre::eyre!("XBL response didn't contain valid user hash"))?;

    Ok(XBLLogin { token, uhs })
}
