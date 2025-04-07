use fuel_data_parser::DataEncoder;
use fuel_streams_types::primitives::*;
use serde::{Deserialize, Serialize};

use crate::infra::record::ToPacket;
#[cfg(any(test, feature = "test-helpers"))]
use crate::{inputs::Input, outputs::Output};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema,
)]
pub struct Utxo {
    pub status: UtxoStatus,
    pub r#type: UtxoType,
    pub tx_id: TxId,
    pub utxo_id: UtxoId,
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub amount: Option<Amount>,
    pub asset_id: Option<AssetId>,
    pub contract_id: Option<ContractId>,
    pub nonce: Option<Nonce>,
}

impl DataEncoder for Utxo {}
impl ToPacket for Utxo {}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockUtxo;

#[cfg(any(test, feature = "test-helpers"))]
impl MockUtxo {
    pub fn from_input(input: &Input) -> Option<Utxo> {
        use fuel_core_types::fuel_types;
        match input {
            Input::Contract(contract) => Some(Utxo {
                status: UtxoStatus::Spent,
                r#type: UtxoType::InputContract,
                utxo_id: contract.utxo_id.clone(),
                tx_id: TxId::random(),
                contract_id: Some(ContractId::new(
                    fuel_types::ContractId::new(*contract.contract_id.0),
                )),
                asset_id: None,
                from: None,
                to: None,
                nonce: None,
                amount: None,
            }),
            Input::Coin(coin) => Some(Utxo {
                status: UtxoStatus::Spent,
                r#type: UtxoType::InputCoin,
                utxo_id: coin.utxo_id.clone(),
                to: Some(coin.owner.clone()),
                amount: Some(coin.amount),
                asset_id: Some(coin.asset_id.clone()),
                from: None,
                nonce: None,
                tx_id: TxId::random(),
                contract_id: None,
            }),
            _ => None,
        }
    }

    pub fn from_output(output: &Output) -> Option<Utxo> {
        match output {
            Output::Coin(coin) => Some(Utxo {
                status: UtxoStatus::Unspent,
                r#type: UtxoType::OutputCoin,
                utxo_id: UtxoId::random(),
                to: Some(coin.to.clone()),
                asset_id: Some(coin.asset_id.clone()),
                amount: Some(coin.amount),
                tx_id: TxId::random(),
                from: None,
                nonce: None,
                contract_id: None,
            }),
            Output::Change(change) => Some(Utxo {
                status: UtxoStatus::Unspent,
                r#type: UtxoType::OutputChange,
                utxo_id: UtxoId::random(),
                to: Some(change.to.clone()),
                amount: Some(change.amount),
                asset_id: Some(change.asset_id.clone()),
                from: None,
                nonce: None,
                tx_id: TxId::random(),
                contract_id: None,
            }),
            Output::Variable(variable) => Some(Utxo {
                status: UtxoStatus::Unspent,
                r#type: UtxoType::OutputVariable,
                utxo_id: UtxoId::random(),
                to: Some(variable.to.clone()),
                amount: Some(variable.amount),
                asset_id: Some(variable.asset_id.clone()),
                from: None,
                nonce: None,
                tx_id: TxId::random(),
                contract_id: None,
            }),
            // Contract and ContractCreated outputs don't create UTXOs
            Output::Contract(_) | Output::ContractCreated(_) => None,
        }
    }
}
