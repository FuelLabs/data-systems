use clap::Parser;
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Undecodable config element: {0}
    UndecodableConfigElement(&'static str),
}

#[derive(Clone, Debug)]
pub struct ApiKeysConfig {
    pub nsize: i32,
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub api_keys: ApiKeysConfig,
    pub db: DbConfig,
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        let cli = crate::cli::Cli::parse();
        Self::from_cli(&cli)
    }

    fn from_cli(cli: &crate::cli::Cli) -> Result<Self, Error> {
        Ok(Config {
            api_keys: ApiKeysConfig { nsize: cli.nkeys },
            db: DbConfig {
                url: cli.db_url.clone(),
            },
        })
    }
}
