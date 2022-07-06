//! "Database" for Hydra
use dashmap::DashMap;
use std::{net::IpAddr, time::Instant};
use trillium_websockets::WebSocketConn;
use uuid::Uuid;

pub struct RuntimeState {
    pub login_attempts: DashMap<UserID, (Instant, u8)>,
    pub auth_sockets: DashMap<Uuid, WebSocketConn>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            login_attempts: DashMap::new(),
            auth_sockets: DashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserID {
    Ipv4([u8; 4]),
    Ipv6([u8; 8]),
}

impl AsRef<[u8]> for UserID {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Ipv4(ref bytes) => bytes,
            Self::Ipv6(ref bytes) => bytes,
        }
    }
}

// This actually is infallible, since ip is always valid
#[allow(clippy::fallible_impl_from)]
impl From<IpAddr> for UserID {
    fn from(addr: IpAddr) -> Self {
        match addr {
            IpAddr::V4(ip) => Self::Ipv4(ip.octets()),
            IpAddr::V6(ip) => Self::Ipv6(ip.octets()[..8].try_into().unwrap()),
        }
    }
}
