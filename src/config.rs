//! Hydra configuration
use std::path::PathBuf;

// TODO: we may want to encrypt one or both of these at compile time
pub const CLIENT_ID: &str = env!("HYDRA_CLIENT_ID");
pub const CLIENT_SECRET: &str = env!("HYDRA_CLIENT_SECRET");

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub db_dir: PathBuf,
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
            env "HYDRA_PORT" => |it| str::parse(&it).ok(),
            default 8080
        );

        let host = config_option!(
            env "HYDRA_HOST",
            default String::from("127.0.0.1")
        );

        let dirs = directories::ProjectDirs::from("com", "Modrinth", "Hydra").unwrap();

        let db_dir = config_option!(
            env "HYDRA_DB" => |it| Some(PathBuf::from(it)),
            default dirs.data_dir().join("db")
        );

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
            db_dir,
            #[cfg(feature = "tls")]
            cert_file,
            #[cfg(feature = "tls")]
            key_file,
        }
    }
}

impl From<&Config> for sled::Config {
    fn from(config: &Config) -> Self {
        Self::default()
            .path(config.db_dir.clone())
            .use_compression(true)
            .mode(sled::Mode::LowSpace)
    }
}
