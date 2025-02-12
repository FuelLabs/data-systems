use std::str::FromStr;

/// FuelStreamsNetworks; shortened to FuelNetworks for brievity and public familiarity
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Default)]
pub enum FuelNetworkUserRole {
    Admin,
    #[default]
    Default,
}

#[derive(Debug, Copy, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FuelNetwork {
    #[default]
    Local,
    Staging,
    Mainnet,
}

impl FromStr for FuelNetwork {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(FuelNetwork::Local),
            "staging" => Ok(FuelNetwork::Staging),
            "mainnet" => Ok(FuelNetwork::Mainnet),
            _ => Err(format!("unknown network: {}", s)),
        }
    }
}

impl std::fmt::Display for FuelNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuelNetwork::Local => write!(f, "local"),
            FuelNetwork::Staging => write!(f, "staging"),
            FuelNetwork::Mainnet => write!(f, "mainnet"),
        }
    }
}

impl FuelNetwork {
    pub fn load_from_env() -> Self {
        match std::env::var("NETWORK").as_deref() {
            Ok("mainnet") => FuelNetwork::Mainnet,
            Ok("staging") => FuelNetwork::Staging,
            _ => FuelNetwork::Local,
        }
    }

    pub fn to_nats_url(&self) -> String {
        match self {
            FuelNetwork::Local => "nats://localhost:4222",
            FuelNetwork::Staging => "nats://stream-staging.fuel.network:4222",
            FuelNetwork::Mainnet => "nats://stream-mainnet.fuel.network:4222",
        }
        .to_string()
    }

    pub fn to_web_url(&self) -> Url {
        match self {
            FuelNetwork::Local => {
                Url::parse("http://localhost:9003").expect("working url")
            }
            FuelNetwork::Staging => {
                Url::parse("https://stream-staging.fuel.network")
                    .expect("working url")
            }
            FuelNetwork::Mainnet => {
                Url::parse("https://stream-mainnet.fuel.network")
                    .expect("working url")
            }
        }
    }

    pub fn to_ws_url(&self) -> Url {
        match self {
            FuelNetwork::Local => {
                Url::parse("ws://0.0.0.0:9003").expect("working url")
            }
            FuelNetwork::Staging => {
                Url::parse("wss://stream-staging.fuel.network")
                    .expect("working url")
            }
            FuelNetwork::Mainnet => {
                Url::parse("wss://stream-mainnet.fuel.network")
                    .expect("working url")
            }
        }
    }
}
