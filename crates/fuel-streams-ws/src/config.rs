use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use displaydoc::Display as DisplayDoc;
use fuel_streams::types::FuelNetwork;
use thiserror::Error;

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Undecodable config element: {0}
    UndecodableConfigElement(&'static str),
}

#[derive(Debug, Default, Clone)]
pub struct S3Config {
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub port: u16,
    pub tls: Option<TlsConfig>,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub jwt_secret: String,
}

#[derive(Clone, Debug)]
pub struct FuelConfig {
    pub network: FuelNetwork,
}

#[derive(Clone, Debug)]
pub struct NatsConfig {
    pub network: FuelNetwork,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub api: ApiConfig,
    pub auth: AuthConfig,
    pub s3: S3Config,
    pub nats: NatsConfig,
    pub fuel: FuelConfig,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let cli = crate::cli::Cli::parse();
        Self::from_cli(&cli)
    }

    fn from_cli(cli: &crate::cli::Cli) -> Result<Self, Error> {
        Ok(Config {
            api: ApiConfig {
                port: cli.api_port,
                tls: None,
            },
            auth: AuthConfig {
                jwt_secret: cli.jwt_secret.clone(),
            },
            nats: NatsConfig {
                network: FuelNetwork::Local,
            },
            s3: S3Config {
                enabled: cli.s3_enabled,
            },
            fuel: FuelConfig {
                network: FuelNetwork::from_str(&cli.network)
                    .map_err(|_| Error::UndecodableConfigElement("NETWORK"))?,
            },
        })
    }
}
