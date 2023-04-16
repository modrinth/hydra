//! Websocket route for Hydra
use std::net::IpAddr;
use crate::{
    db::{RuntimeState, UserID},
    templates::messages,
};
use actix_web::{HttpRequest, HttpResponse, web, get};
use actix_web::web::Payload;
use actix_ws::{Closed, Session};
use uuid::Uuid;

#[get("/")]
pub async fn route(req: HttpRequest, body: Payload, db: web::Data<RuntimeState>) -> Result<HttpResponse, actix_web::Error> {
    let (res, session, _msg_stream) = actix_ws::handle(&req, body)?;
    let _ = sock(req, session, db).await;

    Ok(res)
}

async fn sock(req: HttpRequest, mut ws_stream: Session, db: web::Data<RuntimeState>) -> Result<(), Closed> {
    let addr = if let Some(addr) = {
        let conn_info =  req.connection_info();
        let info = conn_info.realip_remote_addr();

        info.and_then(|x| x.parse::<IpAddr>().ok())
    } {
        addr
    } else {
        ws_stream.text(messages::Error::render_string("Could not determine IP address of connector!"))
            .await?;

        ws_stream.close(None).await?;
        return Ok(());
    };

    let id = UserID::from(addr);
    if db.rate_limit(id) {
        ws_stream.text(messages::Error::render_string(&format!(
            "Rate limit exceeded for IP {addr}"
        )))
            .await?;
        ws_stream.close(None).await?;
        return Ok(());
    }

    let conn_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, id.as_ref());
    ws_stream.text(
        serde_json::json!({
            "login_code": conn_id
            .as_hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
        }).to_string()
    )
        .await?;

    if let Some(mut old_conn) = db.auth_sockets.insert(conn_id, ws_stream) {
        old_conn.text(messages::Error::render_string("New connection created from this address")).await?;
        old_conn.close(None).await?;
    }

    Ok(())
}