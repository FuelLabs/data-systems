use fuel_streams_types::primitives::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Utxo {
    pub utxo_id: UtxoId,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub nonce: Option<Nonce>,
    pub data: Option<HexData>,
    pub amount: Option<u64>,
    pub tx_id: TxId,
}

#[derive(Debug, Clone, Default)]
pub enum UtxoType {
    Contract,
    Coin,
    #[default]
    Message,
}

impl UtxoType {
    fn as_str(&self) -> &'static str {
        match self {
            UtxoType::Contract => "contract",
            UtxoType::Coin => "coin",
            UtxoType::Message => "message",
        }
    }
}

impl std::fmt::Display for UtxoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for UtxoType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s == Self::Contract.as_str() => Ok(Self::Contract),
            s if s == Self::Coin.as_str() => Ok(Self::Coin),
            s if s == Self::Message.as_str() => Ok(Self::Message),
            _ => Err(format!("Invalid UTXO type: {s}")),
        }
    }
}
