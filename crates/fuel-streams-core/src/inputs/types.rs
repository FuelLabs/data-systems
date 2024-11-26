use crate::types::*;

// Input enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
                tx_pointer: input.tx_pointer,
                utxo_id: input.utxo_id,
            }),
            FuelCoreInput::CoinSigned(input) => Input::Coin(InputCoin {
                amount: input.amount,
                asset_id: input.asset_id.into(),
                owner: input.owner.into(),
                predicate: HexString::default(),
                predicate_data: HexString::default(),
                predicate_gas_used: 0,
                tx_pointer: input.tx_pointer,
                utxo_id: input.utxo_id,
                witness_index: input.witness_index,
            }),
            FuelCoreInput::CoinPredicate(input) => Input::Coin(InputCoin {
                amount: input.amount,
                asset_id: input.asset_id.into(),
                owner: input.owner.into(),
                predicate: input.predicate.as_slice().into(),
                predicate_data: input.predicate_data.as_slice().into(),
                predicate_gas_used: input.predicate_gas_used,
                tx_pointer: input.tx_pointer,
                utxo_id: input.utxo_id,
                witness_index: 0,
            }),
            FuelCoreInput::MessageCoinSigned(input) => {
                Input::Message(InputMessage {
                    amount: input.amount,
                    data: HexString::default(),
                    nonce: input.nonce.into(),
                    predicate: HexString::default(),
                    predicate_data: HexString::default(),
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
                    predicate_data: input.predicate_data.as_slice().into(),
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
                    predicate_data: HexString::default(),
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
                    predicate_data: input.predicate_data.as_slice().into(),
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
#[serde(rename_all = "camelCase")]
pub struct InputCoin {
    pub amount: u64,
    pub asset_id: AssetId,
    pub owner: Address,
    pub predicate: HexString,
    pub predicate_data: HexString,
    pub predicate_gas_used: u64,
    pub tx_pointer: FuelCoreTxPointer,
    pub utxo_id: FuelCoreUtxoId,
    pub witness_index: u16,
}

// InputContract type
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputContract {
    pub balance_root: Bytes32,
    pub contract_id: Bytes32,
    pub state_root: Bytes32,
    pub tx_pointer: FuelCoreTxPointer,
    pub utxo_id: FuelCoreUtxoId,
}

impl From<&FuelCoreInputContract> for InputContract {
    fn from(input: &FuelCoreInputContract) -> Self {
        InputContract {
            balance_root: input.balance_root.into(),
            contract_id: input.contract_id.into(),
            state_root: input.state_root.into(),
            tx_pointer: input.tx_pointer,
            utxo_id: input.utxo_id,
        }
    }
}

// InputMessage type
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMessage {
    pub amount: u64,
    pub data: HexString,
    pub nonce: Nonce,
    pub predicate: HexString,
    pub predicate_data: HexString,
    pub predicate_gas_used: u64,
    pub recipient: Address,
    pub sender: Address,
    pub witness_index: u16,
}
