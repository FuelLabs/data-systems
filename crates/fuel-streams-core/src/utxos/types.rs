use fuel_core_types::{
    fuel_tx::{
        input::message::compute_message_id,
        Address,
        Bytes32,
        MessageId,
        UtxoId,
    },
    fuel_types::Nonce,
};
// ------------------------------------------------------------------------
// Utxos
// ------------------------------------------------------------------------
pub use fuel_core_types::{
    fuel_tx::{Receipt, Transaction, UniqueIdentifier},
    services::txpool::TransactionStatus as FuelCoreTransactionStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub id: UtxoId,
    pub sender: Option<Address>,
    pub recipient: Option<Address>,
    pub nonce: Option<Nonce>,
    pub data: Option<Vec<u8>>,
    pub amount: Option<u64>,
    pub tx_id: Bytes32,
}

impl Utxo {
    pub fn new(
        id: UtxoId,
        sender: Option<Address>,
        recipient: Option<Address>,
        nonce: Option<Nonce>,
        data: Option<Vec<u8>>,
        amount: Option<u64>,
        tx_id: Bytes32,
    ) -> Self {
        Self {
            id,
            sender,
            recipient,
            nonce,
            data,
            amount,
            tx_id,
        }
    }
}

impl Utxo {
    pub fn compute_hash(&self) -> MessageId {
        match (
            self.sender.as_ref(),
            self.recipient.as_ref(),
            self.nonce.as_ref(),
            self.amount.as_ref(),
            self.data.as_ref(),
        ) {
            (
                Some(sender),
                Some(recipient),
                Some(nonce),
                Some(amount),
                Some(data),
            ) => compute_message_id(sender, recipient, nonce, *amount, data),
            _ => MessageId::new(*self.tx_id),
        }
    }
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
