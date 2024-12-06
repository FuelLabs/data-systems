#[derive(Debug, Copy, Clone, Default, clap::ValueEnum)]
pub enum FuelNetwork {
    Local,
    #[default]
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
        match std::env::var("FUEL_NETWORK").as_deref() {
            Ok("local") => FuelNetwork::Local,
            Ok("testnet") => FuelNetwork::Testnet,
            Ok("mainnet") => FuelNetwork::Mainnet,
            _ => FuelNetwork::Testnet,
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
}
