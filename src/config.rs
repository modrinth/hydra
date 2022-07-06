//! Hydra configuration
#[allow(unused)]
use std::path::PathBuf;
use std::time::Duration;

// TODO: we may want to encrypt one or both of these at compile time
pub const CLIENT_ID: &str = env!("HYDRA_CLIENT_ID");
pub const CLIENT_SECRET: &str = env!("HYDRA_CLIENT_SECRET");

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub rate_limit: u8,
    pub rate_limit_expires: Duration,
    #[cfg(feature = "tls")]
    pub cert_file: PathBuf,
    #[cfg(feature = "tls")]
    pub key_file: PathBuf,
}

macro_rules! config_option {
    (env $env:literal $(=> $transform:expr)?, default $default:expr) => {
        std::env::var(String::from($env))
            .ok()
            $(.and_then($transform))?
            .unwrap_or($default)
    }
}

impl Config {
    pub fn init() -> Self {
        let port = config_option!(
            env "HYDRA_PORT" => |it| it.parse::<u16>().ok(),
            default 8080
        );

        let host = config_option!(
            env "HYDRA_HOST",
            default String::from("127.0.0.1")
        );

        let rate_limit = config_option!(
            env "HYDRA_RATE_LIMIT" => |it| it.parse::<u8>().ok(),
            default 10
        );

        let rate_limit_expires = config_option!(
            env "HYDRA_RATE_LIMIT_EXPIRES" => |it| {
                let minutes = it.parse::<u64>().ok()?;
                Some(Duration::from_secs(minutes * 60))
            },
            default Duration::from_secs(30 * 60)
        );

        #[allow(unused_variables)]
        let dirs =
            directories::ProjectDirs::from("com", "Modrinth", "Hydra").unwrap();

        #[cfg(feature = "tls")]
        let cert_file = config_option!(
            env "HYDRA_CERT" => |it| Some(PathBuf::from(it)),
            default dirs.config_dir().join("cert.pem")
        );

        #[cfg(feature = "tls")]
        let key_file = config_option!(
            env "HYDRA_KEY" => |it| Some(PathBuf::from(it)),
            default dirs.config_dir().join("key.pem")
        );

        Self {
            port,
            host,
            rate_limit,
            rate_limit_expires,
            #[cfg(feature = "tls")]
            cert_file,
            #[cfg(feature = "tls")]
            key_file,
        }
    }
}
