/// FuelStreamsNetworks; shortened to FuelNetworks for brievity and public familiarity

#[derive(Debug, Clone, Default)]
pub enum FuelNetworkUserRole {
    Admin,
    #[default]
    Default,
}

#[derive(Debug, Copy, Clone, Default, clap::ValueEnum)]
pub enum FuelNetwork {
    #[default]
    Local,
    Testnet,
    Mainnet,
}

impl std::fmt::Display for FuelNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuelNetwork::Local => write!(f, "local"),
            FuelNetwork::Testnet => write!(f, "testnet"),
            FuelNetwork::Mainnet => write!(f, "mainnet"),
        }
    }
}

impl FuelNetwork {
    pub fn load_from_env() -> Self {
        match std::env::var("NETWORK").as_deref() {
            Ok("testnet") => FuelNetwork::Testnet,
            Ok("mainnet") => FuelNetwork::Mainnet,
            _ => FuelNetwork::Local,
        }
    }

    pub fn to_nats_url(&self) -> String {
        match self {
            FuelNetwork::Local => "nats://localhost:4222",
            FuelNetwork::Testnet => "nats://stream-testnet.fuel.network:4222",
            FuelNetwork::Mainnet => "nats://stream.fuel.network:4222",
        }
        .to_string()
    }

    pub fn to_s3_url(&self) -> String {
        match self {
            FuelNetwork::Local => "http://localhost:4566",
            FuelNetwork::Testnet | FuelNetwork::Mainnet => {
                "https://fuel_streams.s3.us-east-1.amazonaws.com"
            }
        }
        .to_string()
    }
}
