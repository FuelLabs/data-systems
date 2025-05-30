use apache_avro::AvroSchema;
use fuel_streams_domains::inputs;
use serde::{Deserialize, Serialize};

use super::TxPointer;
use crate::helpers::AvroBytes;

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct InputContract {
    #[avro(rename = "balanceRoot")]
    pub balance_root: Option<AvroBytes>,
    #[avro(rename = "contractId")]
    pub contract_id: Option<AvroBytes>,
    #[avro(rename = "stateRoot")]
    pub state_root: Option<AvroBytes>,
    #[avro(rename = "txPointer")]
    pub tx_pointer: Option<TxPointer>,
    #[avro(rename = "utxoId")]
    pub utxo_id: Option<String>,
}

impl InputContract {
    pub fn new(input: &inputs::InputContract) -> Self {
        Self {
            balance_root: Some(input.balance_root.clone().into()),
            contract_id: Some(input.contract_id.clone().into()),
            state_root: Some(input.state_root.clone().into()),
            tx_pointer: Some((&input.tx_pointer).into()),
            utxo_id: Some(input.utxo_id.to_string()),
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct InputCoin {
    pub amount: Option<i64>,
    #[avro(rename = "assetId")]
    pub asset_id: Option<AvroBytes>,
    pub owner: Option<AvroBytes>,
    pub predicate: Option<AvroBytes>,
    #[avro(rename = "predicateData")]
    pub predicate_data: Option<AvroBytes>,
    #[avro(rename = "predicateGasUsed")]
    pub predicate_gas_used: Option<i64>,
    #[avro(rename = "txPointer")]
    pub tx_pointer: Option<TxPointer>,
    #[avro(rename = "utxoId")]
    pub utxo_id: Option<String>,
    #[avro(rename = "witnessIndex")]
    pub witness_index: Option<i64>,
}

impl InputCoin {
    pub fn new(input: &inputs::InputCoin) -> Self {
        Self {
            amount: Some(input.amount.0 as i64),
            asset_id: Some(input.asset_id.clone().into()),
            owner: Some(input.owner.clone().into()),
            predicate: Some(input.predicate.clone().into()),
            predicate_data: Some(input.predicate_data.clone().into()),
            predicate_gas_used: Some(input.predicate_gas_used.0 as i64),
            tx_pointer: Some((&input.tx_pointer).into()),
            utxo_id: Some(input.utxo_id.to_string()),
            witness_index: Some(input.witness_index as i64),
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct InputMessage {
    pub amount: Option<i64>,
    pub data: Option<AvroBytes>,
    pub nonce: Option<AvroBytes>,
    pub predicate: Option<AvroBytes>,
    #[avro(rename = "predicateLength")]
    pub predicate_length: Option<i64>,
    #[avro(rename = "predicateData")]
    pub predicate_data: Option<AvroBytes>,
    #[avro(rename = "predicateGasUsed")]
    pub predicate_gas_used: Option<i64>,
    #[avro(rename = "predicateDataLength")]
    pub predicate_data_length: Option<i64>,
    pub recipient: Option<AvroBytes>,
    pub sender: Option<AvroBytes>,
    #[avro(rename = "witnessIndex")]
    pub witness_index: Option<i64>,
}

impl InputMessage {
    pub fn new(input: &inputs::InputMessage) -> Self {
        Self {
            amount: Some(input.amount.0 as i64),
            data: Some(input.data.clone().into()),
            nonce: Some(input.nonce.clone().into()),
            predicate: Some(input.predicate.clone().into()),
            predicate_length: Some(input.predicate_length as i64),
            predicate_data: Some(input.predicate_data.clone().into()),
            predicate_gas_used: Some(input.predicate_gas_used.0 as i64),
            predicate_data_length: Some(input.predicate_data_length as i64),
            recipient: Some(input.recipient.clone().into()),
            sender: Some(input.sender.clone().into()),
            witness_index: Some(input.witness_index as i64),
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    #[avro(rename = "contractInputs")]
    pub contract_inputs: Option<Vec<InputContract>>,
    #[avro(rename = "coinInputs")]
    pub coin_inputs: Option<Vec<InputCoin>>,
    #[avro(rename = "messageInputs")]
    pub message_inputs: Option<Vec<InputMessage>>,
    #[avro(rename = "inputTypes")]
    pub input_types: Option<Vec<String>>,
}

impl Inputs {
    pub fn new(inputs: &[inputs::Input]) -> Self {
        let mut contract_inputs = Vec::new();
        let mut coin_inputs = Vec::new();
        let mut message_inputs = Vec::new();
        let mut input_types = Vec::new();

        for input in inputs {
            match input {
                inputs::Input::Contract(contract) => {
                    contract_inputs.push(InputContract::new(contract));
                    input_types.push("contract".to_string());
                }
                inputs::Input::Coin(coin) => {
                    coin_inputs.push(InputCoin::new(coin));
                    input_types.push("coin".to_string());
                }
                inputs::Input::Message(message) => {
                    message_inputs.push(InputMessage::new(message));
                    input_types.push("message".to_string());
                }
            }
        }

        Self {
            contract_inputs: Some(contract_inputs),
            coin_inputs: Some(coin_inputs),
            message_inputs: Some(message_inputs),
            input_types: Some(input_types),
        }
    }
}

#[cfg(test)]
mod tests {
    use apache_avro::AvroSchema;
    use fuel_streams_domains::inputs::types::MockInput;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{write_schema_files, AvroParser};

    fn test_input_serialization(parser: AvroParser, avro_input: Inputs) {
        let ser = serde_json::to_vec(&avro_input).unwrap();
        let deser = serde_json::from_slice::<Inputs>(&ser).unwrap();
        assert_eq!(avro_input, deser);

        let mut avro_writer = parser.writer_with_schema::<Inputs>().unwrap();
        avro_writer.append(&avro_input).unwrap();
        let serialized = avro_writer.into_inner().unwrap();
        let deserialized = parser
            .reader_with_schema::<Inputs>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(avro_input, deserialized[0]);
    }

    #[test]
    fn test_avro_input_contract() {
        let parser = AvroParser::default();
        let input = MockInput::contract();
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_input_coin_signed() {
        let parser = AvroParser::default();
        let input = MockInput::coin_signed(None);
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_input_coin_predicate() {
        let parser = AvroParser::default();
        let input = MockInput::coin_predicate();
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_input_message_coin_signed() {
        let parser = AvroParser::default();
        let input = MockInput::message_coin_signed();
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_input_message_coin_predicate() {
        let parser = AvroParser::default();
        let input = MockInput::message_coin_predicate();
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_input_message_data_signed() {
        let parser = AvroParser::default();
        let input = MockInput::message_data_signed();
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_input_message_data_predicate() {
        let parser = AvroParser::default();
        let input = MockInput::message_data_predicate();
        let avro_input = Inputs::new(&[input]);
        test_input_serialization(parser, avro_input);
    }

    #[test]
    fn test_avro_inputs_all() {
        let parser = AvroParser::default();
        let inputs = MockInput::all();
        let avro_input = Inputs::new(&inputs);
        test_input_serialization(parser, avro_input);
    }

    #[tokio::test]
    async fn write_input_schemas() {
        let schemas = [
            ("input_contract.json", InputContract::get_schema()),
            ("input_coin.json", InputCoin::get_schema()),
            ("input_message.json", InputMessage::get_schema()),
            ("inputs.json", Inputs::get_schema()),
        ];

        write_schema_files(&schemas).await;
    }
}
