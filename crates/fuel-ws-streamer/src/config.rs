use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use confy::ConfyError;
use displaydoc::Display as DisplayDoc;
use serde::{Deserialize, Deserializer};
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Open config file: {0}
    OpenConfig(std::io::Error),
    /// Failed to parse config: {0}
    ParseConfig(toml::de::Error),
    /// Failed to parse config as utf-8: {0}
    ParseUtf8(std::string::FromUtf8Error),
    /// Failed to read config file: {0}
    ReadConfig(std::io::Error),
    /// Failed to read config metadata: {0}
    ReadMeta(std::io::Error),
    /// Failed to read env config: {0}
    Confy(ConfyError),
    /// Missing config element: {0}
    MissingConfigElement(&'static str),
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct S3Config {
    pub enabled: bool,
    pub region: String,
    pub bucket: String,
    pub endpoint: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ApiConfig {
    pub port: u16,
    pub tls: Option<TlsConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AuthConfig {
    pub jwt_secret: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    pub api: ApiConfig,
    pub auth: AuthConfig,
    pub s3: S3Config,
    pub nats: NatsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api: ApiConfig {
                port: 9003,
                tls: None,
            },
            auth: AuthConfig {
                jwt_secret: String::new(),
            },
            nats: NatsConfig { url: String::new() },
            s3: S3Config {
                enabled: false,
                region: String::new(),
                bucket: String::new(),
                endpoint: String::new(),
            },
        }
    }
}

#[allow(dead_code)]
fn deserialize_duration_from_usize<'de, D>(
    deserializer: D,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

#[allow(dead_code)]
fn deserialize_duration_option<'de, D>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds: Option<u64> = Option::deserialize(deserializer)?;
    if seconds.is_none() {
        return Ok(None);
    }
    Ok(seconds.map(Duration::from_secs))
}

impl Config {
    pub async fn new(path: impl AsRef<Path> + Send) -> Result<Self, Error> {
        read_to_string(path).await?.parse()
    }

    pub fn from_envs(&mut self) {
        // // ----------------------DB--------------------------------
        // let db_user = std::env::var("POSTGRES_USER").ok();
        // if let Some(db_user) = db_user {
        //     self.db.username = db_user;
        // }

        // let db_pwd = std::env::var("POSTGRES_PASSWORD").ok();
        // if let Some(db_pwd) = db_pwd {
        //     self.db.password = Some(db_pwd);
        // }

        // let db_name = std::env::var("POSTGRES_DB").ok();
        // if let Some(db_name) = db_name {
        //     self.db.database = Some(db_name);
        // }

        // let db_host = std::env::var("POSTGRES_HOST").ok();
        // if let Some(db_host) = db_host {
        //     self.db.host = db_host;
        // }

        // let db_port = std::env::var("POSTGRES_PORT")
        //     .ok()
        //     .map(|p| p.parse::<u16>().ok())
        //     .flatten();
        // if let Some(db_port) = db_port {
        //     self.db.port = db_port;
        // }

        // // ----------------------AMQP--------------------------------
        // let amqp_host = std::env::var("RABBITMQ_HOST").ok();
        // if let Some(amqp_host) = amqp_host {
        //     self.amqp.host = amqp_host;
        // }

        // let amqp_port = std::env::var("RABBITMQ_PORT")
        //     .ok()
        //     .map(|p| p.parse::<u16>().ok())
        //     .flatten();
        // if let Some(amqp_port) = amqp_port {
        //     self.amqp.port = amqp_port;
        // }

        // let amqp_user = std::env::var("RABBITMQ_USER").ok();
        // if let Some(amqp_user) = amqp_user {
        //     self.amqp.username = amqp_user;
        // }

        // let amqp_pwd = std::env::var("RABBITMQ_PASSWORD").ok();
        // if let Some(amqp_pwd) = amqp_pwd {
        //     self.amqp.password = amqp_pwd;
        // }

        // // ----------------------JWT--------------------------------
        // if let Some(jwt_secret) = std::env::var("JWT_SECRET").ok() {
        //     self.auth.jwt_secret = jwt_secret;
        // }

        // ----------------------REDIS--------------------------------
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::ParseConfig)
    }
}

async fn read_to_string(
    path: impl AsRef<Path> + Send,
) -> Result<String, Error> {
    let mut file = File::open(path).await.map_err(Error::OpenConfig)?;
    let meta = file.metadata().await.map_err(Error::ReadMeta)?;
    let mut contents =
        Vec::with_capacity(usize::try_from(meta.len()).unwrap_or(0));
    file.read_to_end(&mut contents)
        .await
        .map_err(Error::ReadConfig)?;
    String::from_utf8(contents).map_err(Error::ParseUtf8)
}
