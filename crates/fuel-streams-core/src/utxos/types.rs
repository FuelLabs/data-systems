use crate::prelude::*;

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
