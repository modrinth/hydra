//! Websocket route for Hydra
use crate::{
    config::Config,
    db::{RuntimeState, UserID},
};
use std::{sync::Arc, time::Instant};
use trillium::{conn_unwrap, Conn};
use trillium_askama::Template;
use trillium_websockets::{websocket, WebSocketConn};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "socket_error.json")]
pub struct WebSocketError<'a> {
    reason: &'a str,
}

#[inline]
pub(super) fn render_error(reason: &str) -> String {
    WebSocketError { reason }.render().unwrap()
}

#[derive(Template)]
#[template(path = "socket_code.json")]
pub struct WebSocketCode<'a> {
    login_code: &'a str,
}

pub fn route() -> impl trillium::Handler {
    (attach_ip, websocket(sock))
}

#[allow(clippy::unused_async)]
pub async fn attach_ip(conn: Conn) -> Conn {
    let ip = conn_unwrap!(conn.inner().peer_ip(), conn);
    conn.with_state(ip)
}

pub async fn sock(mut conn: WebSocketConn) {
    let addr = if let Some(addr) = conn.state::<std::net::IpAddr>() {
        *addr
    } else {
        conn.send_string(render_error(
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

    let (last_req_time, rate) = state
        .login_attempts
        .get(&id)
        .map_or((Instant::now(), 0), |it| *it.value());

    match (last_req_time, rate) {
        (expired, _) if expired.elapsed() > config.rate_limit_expires => {
            state.login_attempts.insert(id, (Instant::now(), 1));
        }
        (_, rate) if rate >= config.rate_limit => {
            conn.send_string(render_error(&format!(
                "Rate limit exceeded for IP {addr}"
            )))
            .await;
            trillium::log_error!(conn.close().await);
            return;
        }
        (_, rate) => {
            state.login_attempts.insert(id, (Instant::now(), rate + 1));
        }
    }

    let conn_id = Uuid::new_v5(&Uuid::NAMESPACE_URL, id.as_ref());
    conn.send_string(
        WebSocketCode {
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
            .send_string(render_error(
                "New connection created from this address",
            ))
            .await;
        trillium::log_error!(old_conn.close().await);
    }
}
