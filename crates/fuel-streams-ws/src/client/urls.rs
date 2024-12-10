use fuel_streams::types::FuelNetwork;
use url::Url;

pub fn get_web_url(network: FuelNetwork) -> Url {
    match network {
        FuelNetwork::Local => {
            Url::parse("http://0.0.0.0:9003").expect("working url")
        }
        FuelNetwork::Testnet => {
            Url::parse("http://0.0.0.0:9003").expect("working url")
        }
        FuelNetwork::Mainnet => {
            Url::parse("http://0.0.0.0:9003").expect("working url")
        }
    }
}

pub fn get_ws_url(network: FuelNetwork) -> Url {
    match network {
        FuelNetwork::Local => {
            Url::parse("ws://0.0.0.0:9003").expect("working url")
        }
        FuelNetwork::Testnet => {
            Url::parse("ws://0.0.0.0:9003").expect("working url")
        }
        FuelNetwork::Mainnet => {
            Url::parse("ws://0.0.0.0:9003").expect("working url")
        }
    }
}
