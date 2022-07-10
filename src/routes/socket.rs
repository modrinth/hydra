//! Websocket route for Hydra
use crate::{
    config::Config,
    db::{RuntimeState, UserID},
    templates::messages,
};
use std::sync::Arc;
use trillium_askama::Template;
use trillium_websockets::{websocket, WebSocketConn};
use uuid::Uuid;

#[inline]
pub fn route() -> impl trillium::Handler {
    websocket(sock)
}

pub async fn sock(mut conn: WebSocketConn) {
    let addr = if let Some(addr) = conn.peer_ip() {
        addr
    } else {
        conn.send_string(messages::Error::render(
            "Could not determine IP address of connector!",
        ))
        .await;

        trillium::log_error!(conn.close().await);
        return;
    };

    let id = UserID::from(addr);
    let (config, state) = (
        conn.take_state::<Arc<Config>>().unwrap(),
        conn.take_state::<Arc<RuntimeState>>().unwrap(),
    );
    if state.rate_limit(&config, id) {
        conn.send_string(messages::Error::render(&format!(
            "Rate limit exceeded for IP {addr}"
        )))
        .await;
        trillium::log_error!(conn.close().await);
        return;
    }

    let conn_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, id.as_ref());
    conn.send_string(
        messages::RateLimitCode {
            login_code: conn_id
                .as_hyphenated()
                .encode_lower(&mut Uuid::encode_buffer()),
        }
        .render()
        .unwrap(),
    )
    .await;

    if let Some(mut old_conn) = state.auth_sockets.insert(conn_id, conn) {
        old_conn
            .send_string(messages::Error::render(
                "New connection created from this address",
            ))
            .await;
        trillium::log_error!(old_conn.close().await);
    }
}
