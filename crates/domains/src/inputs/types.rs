use fuel_data_parser::DataEncoder;
use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

use crate::infra::record::ToPacket;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
    derive_more::IsVariant,
)]
#[serde(tag = "type")]
pub enum Input {
    Contract(InputContract),
    Coin(InputCoin),
    Message(InputMessage),
}

impl DataEncoder for Input {}
impl ToPacket for Input {}

impl From<&FuelCoreInput> for Input {
    fn from(input: &FuelCoreInput) -> Self {
        match input {
            FuelCoreInput::Contract(input) => Input::Contract(InputContract {
                balance_root: input.balance_root.into(),
                contract_id: input.contract_id.into(),
                state_root: input.state_root.into(),
                tx_pointer: input.tx_pointer.into(),
                utxo_id: input.utxo_id.into(),
            }),
            FuelCoreInput::CoinSigned(input) => Input::Coin(InputCoin {
                amount: input.amount.into(),
                asset_id: input.asset_id.into(),
                owner: input.owner.into(),
                predicate: HexData::random(),
                predicate_data: HexData::random(),
                predicate_gas_used: 0.into(),
                tx_pointer: input.tx_pointer.into(),
                utxo_id: input.utxo_id.into(),
                witness_index: input.witness_index,
            }),
            FuelCoreInput::CoinPredicate(input) => Input::Coin(InputCoin {
                amount: input.amount.into(),
                asset_id: input.asset_id.into(),
                owner: input.owner.into(),
                predicate: HexData(input.predicate.as_slice().into()),
                predicate_data: HexData(input.predicate_data.as_slice().into()),
                predicate_gas_used: input.predicate_gas_used.into(),
                tx_pointer: input.tx_pointer.into(),
                utxo_id: input.utxo_id.into(),
                witness_index: 0,
            }),
            FuelCoreInput::MessageCoinSigned(input) => {
                Input::Message(InputMessage {
                    amount: input.amount.into(),
                    data: HexData::random(),
                    nonce: input.nonce.into(),
                    predicate: HexData::random(),
                    predicate_length: 0,
                    predicate_data: HexData::random(),
                    predicate_data_length: 0,
                    predicate_gas_used: 0.into(),
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: 0,
                })
            }
            FuelCoreInput::MessageCoinPredicate(input) => {
                Input::Message(InputMessage {
                    amount: input.amount.into(),
                    data: HexData::random(),
                    nonce: input.nonce.into(),
                    predicate: HexData(input.predicate.as_slice().into()),
                    predicate_length: input.predicate.as_slice().len(),
                    predicate_data: HexData(
                        input.predicate_data.as_slice().into(),
                    ),
                    predicate_data_length: input
                        .predicate_data
                        .as_slice()
                        .len(),
                    predicate_gas_used: input.predicate_gas_used.into(),
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: 0,
                })
            }
            FuelCoreInput::MessageDataSigned(input) => {
                Input::Message(InputMessage {
                    amount: input.amount.into(),
                    data: HexData(input.data.as_slice().into()),
                    nonce: input.nonce.into(),
                    predicate: HexData::random(),
                    predicate_length: 0,
                    predicate_data: HexData::random(),
                    predicate_data_length: 0,
                    predicate_gas_used: 0.into(),
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: input.witness_index,
                })
            }
            FuelCoreInput::MessageDataPredicate(input) => {
                Input::Message(InputMessage {
                    amount: input.amount.into(),
                    data: HexData(input.data.as_slice().into()),
                    nonce: input.nonce.into(),
                    predicate: HexData(input.predicate.as_slice().into()),
                    predicate_length: input.predicate.as_slice().len(),
                    predicate_data: HexData(
                        input.predicate_data.as_slice().into(),
                    ),
                    predicate_data_length: input
                        .predicate_data
                        .as_slice()
                        .len(),
                    predicate_gas_used: input.predicate_gas_used.into(),
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: 0,
                })
            }
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Input::Contract(InputContract::default())
    }
}

// InputCoin type
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct InputCoin {
    pub amount: Amount,
    pub asset_id: AssetId,
    pub owner: Address,
    pub predicate: HexData,
    pub predicate_data: HexData,
    pub predicate_gas_used: GasAmount,
    pub tx_pointer: TxPointer,
    pub utxo_id: UtxoId,
    pub witness_index: u16,
}

// InputContract type
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct InputContract {
    pub balance_root: Bytes32,
    pub contract_id: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: TxPointer,
    pub utxo_id: UtxoId,
}

impl From<&FuelCoreInputContract> for InputContract {
    fn from(input: &FuelCoreInputContract) -> Self {
        InputContract {
            balance_root: input.balance_root.into(),
            contract_id: input.contract_id.into(),
            state_root: input.state_root.into(),
            tx_pointer: input.tx_pointer.into(),
            utxo_id: input.utxo_id.into(),
        }
    }
}

// InputMessage type
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub struct InputMessage {
    pub amount: Amount,
    pub data: HexData,
    pub nonce: Nonce,
    pub predicate: HexData,
    pub predicate_length: usize,
    pub predicate_data: HexData,
    pub predicate_gas_used: GasAmount,
    pub predicate_data_length: usize,
    pub recipient: Address,
    pub sender: Address,
    pub witness_index: u16,
}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockInput;
#[cfg(any(test, feature = "test-helpers"))]
impl MockInput {
    const VALID_PREDICATE_BYTECODE: &str = "1a403000504100301a445000ba49000032400481504100205d490000504100083240048220451300524510044a440000cf534ed3e0f17779f12866863001e53beb68e87621308fbe7f575652dba0dc000000000000000108f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad0700000000000000010000000000002710666c984d4c0aa70abb14a6d6e7693fc5bda8275d6c6716c8bcae33b3c21a1dfb6fd333a74ac52ca7d50d7e768996acd0fb339fcc8a109796b2c55d2f341631d3a0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812edfda4cd39004d68b93c8be7a82f67c18661345e0b8e03a479a9eb4118277c2f190d67a87f1def93ab95e5d940d1534e2d9fed411ba78c9add53930d5b567d3b2cccccccccccc00020000000000000000000000000000000000000000000000000000000000000000000000000000158c0000000000000cf4";

    pub fn contract() -> Input {
        Input::Contract(InputContract {
            balance_root: Bytes32::random(),
            contract_id: Bytes32::random(),
            state_root: Bytes32::random(),
            tx_pointer: TxPointer::random(),
            utxo_id: UtxoId::random(),
        })
    }

    pub fn coin_signed(utxo_id: Option<UtxoId>) -> Input {
        Input::Coin(InputCoin {
            amount: 100.into(),
            asset_id: AssetId::random(),
            owner: Address::random(),
            predicate: HexData::random(),
            predicate_data: HexData::random(),
            predicate_gas_used: 0.into(),
            tx_pointer: TxPointer::random(),
            utxo_id: utxo_id.unwrap_or(UtxoId::random()),
            witness_index: 0,
        })
    }

    pub fn coin_predicate() -> Input {
        let predicate = hex::decode(Self::VALID_PREDICATE_BYTECODE).unwrap();
        Input::Coin(InputCoin {
            amount: 100.into(),
            asset_id: AssetId::random(),
            owner: Address::random(),
            predicate: HexData(predicate.into()),
            predicate_data: HexData::random(),
            predicate_gas_used: 1000.into(),
            tx_pointer: TxPointer::random(),
            utxo_id: UtxoId::random(),
            witness_index: 0,
        })
    }

    pub fn message_coin_signed() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData::random(),
            nonce: Nonce::random(),
            predicate: HexData::random(),
            predicate_length: 0,
            predicate_data: HexData::random(),
            predicate_data_length: 0,
            predicate_gas_used: 0.into(),
            recipient: Address::random(),
            sender: Address::random(),
            witness_index: 0,
        })
    }

    pub fn message_coin_predicate() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData::random(),
            nonce: Nonce::random(),
            predicate: HexData::random(),
            predicate_length: 3,
            predicate_data: HexData::random(),
            predicate_data_length: 3,
            predicate_gas_used: 1000.into(),
            recipient: Address::random(),
            sender: Address::random(),
            witness_index: 0,
        })
    }

    pub fn message_data_signed() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData::random(),
            nonce: Nonce::random(),
            predicate: HexData::random(),
            predicate_length: 0,
            predicate_data: HexData::random(),
            predicate_data_length: 0,
            predicate_gas_used: 0.into(),
            recipient: Address::random(),
            sender: Address::random(),
            witness_index: 0,
        })
    }

    pub fn message_data_predicate() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData::random(),
            nonce: Nonce::random(),
            predicate: HexData::random(),
            predicate_length: 3,
            predicate_data: HexData::random(),
            predicate_data_length: 3,
            predicate_gas_used: 1000.into(),
            recipient: Address::random(),
            sender: Address::random(),
            witness_index: 0,
        })
    }

    pub fn all() -> Vec<Input> {
        vec![
            Self::contract(),
            Self::coin_signed(None),
            Self::coin_predicate(),
            Self::message_coin_signed(),
            Self::message_coin_predicate(),
            Self::message_data_signed(),
            Self::message_data_predicate(),
        ]
    }
}
