use fuel_streams_types::{fuel_core::*, primitives::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "type")]
pub enum Input {
    Contract(InputContract),
    Coin(InputCoin),
    Message(InputMessage),
}

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
                predicate: HexData::default(),
                predicate_data: HexData::default(),
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
                    data: HexData::default(),
                    nonce: input.nonce.into(),
                    predicate: HexData::default(),
                    predicate_length: 0,
                    predicate_data: HexData::default(),
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
                    data: HexData::default(),
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
                    predicate: HexData::default(),
                    predicate_length: 0,
                    predicate_data: HexData::default(),
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

impl InputMessage {
    pub fn compute_message_id(&self) -> MessageId {
        let hasher = fuel_core_types::fuel_crypto::Hasher::default()
            .chain(self.sender.as_ref())
            .chain(self.recipient.as_ref())
            .chain(self.nonce.as_ref())
            .chain(self.amount.to_be_bytes())
            .chain(self.data.0.as_ref());

        (*hasher.finalize()).into()
    }

    pub fn computed_utxo_id(&self) -> UtxoId {
        let message_id = self.compute_message_id();
        UtxoId {
            tx_id: Bytes32::from(message_id),
            output_index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum InputType {
    Contract,
    Coin,
    Message,
}

impl InputType {
    pub fn as_str(&self) -> &str {
        match self {
            InputType::Contract => "contract",
            InputType::Coin => "coin",
            InputType::Message => "message",
        }
    }
}

impl std::fmt::Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for InputType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contract" => Ok(InputType::Contract),
            "coin" => Ok(InputType::Coin),
            "message" => Ok(InputType::Message),
            _ => Err(format!("Invalid input type: {}", s)),
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MockInput;
#[cfg(any(test, feature = "test-helpers"))]
impl MockInput {
    pub fn contract() -> Input {
        Input::Contract(InputContract {
            balance_root: Bytes32::default(),
            contract_id: Bytes32::default(),
            state_root: Bytes32::default(),
            tx_pointer: TxPointer::default(),
            utxo_id: UtxoId::default(),
        })
    }

    pub fn coin_signed() -> Input {
        Input::Coin(InputCoin {
            amount: 100.into(),
            asset_id: AssetId::default(),
            owner: Address::default(),
            predicate: HexData::default(),
            predicate_data: HexData::default(),
            predicate_gas_used: 0.into(),
            tx_pointer: TxPointer::default(),
            utxo_id: UtxoId::default(),
            witness_index: 0,
        })
    }

    pub fn coin_predicate() -> Input {
        Input::Coin(InputCoin {
            amount: 100.into(),
            asset_id: AssetId::default(),
            owner: Address::default(),
            predicate: HexData(vec![1, 2, 3].into()),
            predicate_data: HexData(vec![4, 5, 6].into()),
            predicate_gas_used: 1000.into(),
            tx_pointer: TxPointer::default(),
            utxo_id: UtxoId::default(),
            witness_index: 0,
        })
    }

    pub fn message_coin_signed() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData::default(),
            nonce: Nonce::default(),
            predicate: HexData::default(),
            predicate_length: 0,
            predicate_data: HexData::default(),
            predicate_data_length: 0,
            predicate_gas_used: 0.into(),
            recipient: Address::default(),
            sender: Address::default(),
            witness_index: 0,
        })
    }

    pub fn message_coin_predicate() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData::default(),
            nonce: Nonce::default(),
            predicate: HexData(vec![1, 2, 3].into()),
            predicate_length: 3,
            predicate_data: HexData(vec![4, 5, 6].into()),
            predicate_data_length: 3,
            predicate_gas_used: 1000.into(),
            recipient: Address::default(),
            sender: Address::default(),
            witness_index: 0,
        })
    }

    pub fn message_data_signed() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData(vec![7, 8, 9].into()),
            nonce: Nonce::default(),
            predicate: HexData::default(),
            predicate_length: 0,
            predicate_data: HexData::default(),
            predicate_data_length: 0,
            predicate_gas_used: 0.into(),
            recipient: Address::default(),
            sender: Address::default(),
            witness_index: 0,
        })
    }

    pub fn message_data_predicate() -> Input {
        Input::Message(InputMessage {
            amount: 100.into(),
            data: HexData(vec![7, 8, 9].into()),
            nonce: Nonce::default(),
            predicate: HexData(vec![1, 2, 3].into()),
            predicate_length: 3,
            predicate_data: HexData(vec![4, 5, 6].into()),
            predicate_data_length: 3,
            predicate_gas_used: 1000.into(),
            recipient: Address::default(),
            sender: Address::default(),
            witness_index: 0,
        })
    }

    pub fn all() -> Vec<Input> {
        vec![
            Self::contract(),
            Self::coin_signed(),
            Self::coin_predicate(),
            Self::message_coin_signed(),
            Self::message_coin_predicate(),
            Self::message_data_signed(),
            Self::message_data_predicate(),
        ]
    }
}
