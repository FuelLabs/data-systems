use std::{
    num::ParseIntError,
    path::{Path, PathBuf},
    str::{FromStr, ParseBoolError},
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
    /// Parse int error: {0}
    ParseInt(ParseIntError),
    /// Parse bool error: {0}
    ParseBool(ParseBoolError),
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
    pub async fn from_path(
        path: impl AsRef<Path> + Send,
    ) -> Result<Self, Error> {
        read_to_string(path).await?.parse()
    }

    pub fn from_envs() -> Result<Self, Error> {
        let mut config = Self::default();

        // ----------------------API--------------------------------
        if let Ok(app_port) = dotenvy::var("API_PORT") {
            config.api.port =
                app_port.parse::<u16>().map_err(Error::ParseInt)?;
        }

        // ----------------------NATS--------------------------------
        if let Ok(nats_url) = dotenvy::var("NATS_URL") {
            config.nats.url = nats_url;
        }

        // ----------------------S3--------------------------------
        if let Ok(s3_enabled) = dotenvy::var("S3_ENABLED") {
            config.s3.enabled =
                s3_enabled.parse::<bool>().map_err(Error::ParseBool)?;
        }
        if let Ok(s3_region) = dotenvy::var("S3_REGION") {
            config.s3.region = s3_region;
        }
        if let Ok(s3_bucket) = dotenvy::var("S3_BUCKET") {
            config.s3.bucket = s3_bucket;
        }
        if let Ok(s3_endpoint) = dotenvy::var("S3_ENDPOINT") {
            config.s3.endpoint = s3_endpoint;
        }

        // ----------------------AUTH--------------------------------
        if let Ok(jwt_secret) = dotenvy::var("JWT_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }
        Ok(config)
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