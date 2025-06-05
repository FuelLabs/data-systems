use std::path::PathBuf;

use clap::Parser;
use displaydoc::Display as DisplayDoc;
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
pub struct DbConfig {
    pub url: String,
    pub read_url: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub api: ApiConfig,
    pub db: DbConfig,
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
            db: DbConfig {
                url: cli.db_url.clone(),
                read_url: cli.db_url_read.clone().unwrap_or(cli.db_url.clone()),
            },
        })
    }
}
