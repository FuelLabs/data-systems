use fuel_core_types::fuel_crypto;

use crate::types::*;

// Input enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
                amount: input.amount,
                asset_id: input.asset_id.into(),
                owner: input.owner.into(),
                predicate: HexString::default(),
                predicate_data: HexString::default(),
                predicate_gas_used: 0,
                tx_pointer: input.tx_pointer.into(),
                utxo_id: input.utxo_id.into(),
                witness_index: input.witness_index,
            }),
            FuelCoreInput::CoinPredicate(input) => Input::Coin(InputCoin {
                amount: input.amount,
                asset_id: input.asset_id.into(),
                owner: input.owner.into(),
                predicate: input.predicate.as_slice().into(),
                predicate_data: input.predicate_data.as_slice().into(),
                predicate_gas_used: input.predicate_gas_used,
                tx_pointer: input.tx_pointer.into(),
                utxo_id: input.utxo_id.into(),
                witness_index: 0,
            }),
            FuelCoreInput::MessageCoinSigned(input) => {
                Input::Message(InputMessage {
                    amount: input.amount,
                    data: HexString::default(),
                    nonce: input.nonce.into(),
                    predicate: HexString::default(),
                    predicate_length: 0,
                    predicate_data: HexString::default(),
                    predicate_data_length: 0,
                    predicate_gas_used: 0,
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: 0,
                })
            }
            FuelCoreInput::MessageCoinPredicate(input) => {
                Input::Message(InputMessage {
                    amount: input.amount,
                    data: HexString::default(),
                    nonce: input.nonce.into(),
                    predicate: input.predicate.as_slice().into(),
                    predicate_length: input.predicate.as_slice().len(),
                    predicate_data: input.predicate_data.as_slice().into(),
                    predicate_data_length: input
                        .predicate_data
                        .as_slice()
                        .len(),
                    predicate_gas_used: input.predicate_gas_used,
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: 0,
                })
            }
            FuelCoreInput::MessageDataSigned(input) => {
                Input::Message(InputMessage {
                    amount: input.amount,
                    data: input.data.as_slice().into(),
                    nonce: input.nonce.into(),
                    predicate: HexString::default(),
                    predicate_length: 0,
                    predicate_data: HexString::default(),
                    predicate_data_length: 0,
                    predicate_gas_used: 0,
                    recipient: input.recipient.into(),
                    sender: input.sender.into(),
                    witness_index: input.witness_index,
                })
            }
            FuelCoreInput::MessageDataPredicate(input) => {
                Input::Message(InputMessage {
                    amount: input.amount,
                    data: input.data.as_slice().into(),
                    nonce: input.nonce.into(),
                    predicate: input.predicate.as_slice().into(),
                    predicate_length: input.predicate.as_slice().len(),
                    predicate_data: input.predicate_data.as_slice().into(),
                    predicate_data_length: input
                        .predicate_data
                        .as_slice()
                        .len(),
                    predicate_gas_used: input.predicate_gas_used,
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InputCoin {
    pub amount: u64,
    pub asset_id: AssetId,
    pub owner: Address,
    pub predicate: HexString,
    pub predicate_data: HexString,
    pub predicate_gas_used: u64,
    pub tx_pointer: TxPointer,
    pub utxo_id: UtxoId,
    pub witness_index: u16,
}

// InputContract type
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InputMessage {
    pub amount: u64,
    pub data: HexString,
    pub nonce: Nonce,
    pub predicate: HexString,
    pub predicate_length: usize,
    pub predicate_data: HexString,
    pub predicate_gas_used: u64,
    pub predicate_data_length: usize,
    pub recipient: Address,
    pub sender: Address,
    pub witness_index: u16,
}

impl InputMessage {
    pub fn compute_message_id(&self) -> MessageId {
        let hasher = fuel_crypto::Hasher::default()
            .chain(self.sender.as_ref())
            .chain(self.recipient.as_ref())
            .chain(self.nonce.as_ref())
            .chain(self.amount.to_be_bytes())
            .chain(self.data.as_ref());

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
