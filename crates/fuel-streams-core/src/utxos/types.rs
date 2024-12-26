use crate::prelude::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Utxo {
    pub utxo_id: UtxoId,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub nonce: Option<Nonce>,
    pub data: Option<HexString>,
    pub amount: Option<u64>,
    pub tx_id: Bytes32,
}

#[derive(Debug, Clone, Default)]
pub enum UtxoType {
    Contract,
    Coin,
    #[default]
    Message,
}

impl std::fmt::Display for UtxoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            UtxoType::Contract => "contract",
            UtxoType::Coin => "coin",
            UtxoType::Message => "message",
        };
        write!(f, "{value}")
    }
}
