use std::{
    num::ParseIntError,
    path::{Path, PathBuf},
    str::{FromStr, ParseBoolError},
    time::Duration,
};

use confy::ConfyError;
use displaydoc::Display as DisplayDoc;
use fuel_streams::types::FuelNetwork;
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
    /// Undecodable config element: {0}
    UndecodableConfigElement(&'static str),
    /// Parse int error: {0}
    ParseInt(ParseIntError),
    /// Parse bool error: {0}
    ParseBool(ParseBoolError),
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct S3Config {
    pub enabled: bool,
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
pub struct FuelConfig {
    pub network: FuelNetwork,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct NatsConfig {
    pub network: FuelNetwork,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    pub api: ApiConfig,
    pub auth: AuthConfig,
    pub s3: S3Config,
    pub nats: NatsConfig,
    pub fuel: FuelConfig,
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
            nats: NatsConfig {
                network: FuelNetwork::Local,
            },
            s3: S3Config { enabled: false },
            fuel: FuelConfig {
                network: FuelNetwork::Local,
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
        if let Ok(app_port) = dotenvy::var("STREAMER_API_PORT") {
            config.api.port =
                app_port.parse::<u16>().map_err(Error::ParseInt)?;
        }

        // ----------------------NATS--------------------------------
        if let Ok(nats_network) = dotenvy::var("NETWORK") {
            config.nats.network = FuelNetwork::from_str(&nats_network)
                .map_err(|_| Error::UndecodableConfigElement("NETWORK"))?;
        }

        // ----------------------S3--------------------------------
        if let Ok(s3_enabled) = dotenvy::var("AWS_S3_ENABLED") {
            config.s3.enabled =
                s3_enabled.parse::<bool>().map_err(Error::ParseBool)?;
        }

        // ----------------------AUTH--------------------------------
        if let Ok(jwt_secret) = dotenvy::var("JWT_AUTH_SECRET") {
            config.auth.jwt_secret = jwt_secret;
        }

        // ----------------------FUEL--------------------------------
        if let Ok(network) = dotenvy::var("NETWORK") {
            config.fuel.network = FuelNetwork::from_str(&network)
                .map_err(|_| Error::UndecodableConfigElement("NETWORK"))?;
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
