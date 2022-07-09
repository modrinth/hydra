//! Hydra configuration
#[allow(unused)]
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub public_url: Url,
    pub client_id: String,
    pub client_secret: String,
    pub rate_limit: u8,
    pub rate_limit_expires: Duration,
    #[cfg(feature = "tls")]
    pub cert_file: PathBuf,
    #[cfg(feature = "tls")]
    pub key_file: PathBuf,
}

macro_rules! config_option {
    (env $env:literal $(=> $transform:expr)?$(, default $default:expr)?) => {
        std::env::var(String::from($env))
            .ok()
            $(.and_then($transform))?
            $(.unwrap_or_else(|| $default))?
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

        let public_url = config_option!(
            env "HYDRA_PUBLIC_URL" => |it| Url::parse(&it).ok(),
            default Url::parse("https://{host}:{port}/").unwrap()
        );

        let client_id = config_option!(
            env "HYDRA_CLIENT_ID"
        )
        .expect("Could not find Hydra client ID!");

        let client_secret = config_option!(
            env "HYDRA_CLIENT_SECRET"
        )
        .expect("Could not find Hydra client secret!");

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
        let (cert_file, key_file) = (
            config_option!(
                env "HYDRA_CERT" => |it| Some(PathBuf::from(it)),
                default dirs.config_dir().join("cert.pem")
            ),
            config_option!(
                env "HYDRA_KEY" => |it| Some(PathBuf::from(it)),
                default dirs.config_dir().join("key.pem")
            ),
        );

        Self {
            port,
            host,
            public_url,
            client_id,
            client_secret,
            rate_limit,
            rate_limit_expires,
            #[cfg(feature = "tls")]
            cert_file,
            #[cfg(feature = "tls")]
            key_file,
        }
    }
}
