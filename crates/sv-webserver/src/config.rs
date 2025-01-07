use std::path::PathBuf;

use clap::Parser;
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Undecodable config element: {0}
    UndecodableConfigElement(&'static str),
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
pub struct NatsConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub api: ApiConfig,
    pub auth: AuthConfig,
    pub db: DbConfig,
    pub nats: NatsConfig,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let cli = crate::cli::Cli::parse();
        Self::from_cli(&cli)
    }

    fn from_cli(cli: &crate::cli::Cli) -> Result<Self, Error> {
        Ok(Config {
            api: ApiConfig {
                port: cli.port,
                tls: None,
            },
            auth: AuthConfig {
                jwt_secret: cli.jwt_secret.clone(),
            },
            nats: NatsConfig {
                url: cli.nats_url.clone(),
            },
            db: DbConfig {
                url: cli.db_url.clone(),
            },
        })
    }
}
