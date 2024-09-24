// ------------------------------------------------------------------------
// Utxos
// ------------------------------------------------------------------------
pub use fuel_core_types::{
    fuel_tx::{Receipt, Transaction, UniqueIdentifier},
    services::txpool::TransactionStatus as FuelCoreTransactionStatus,
};

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
