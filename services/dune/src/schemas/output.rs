use apache_avro::AvroSchema;
use fuel_streams_domains::outputs;
use serde::{Deserialize, Serialize};

use crate::helpers::AvroBytes;

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputCoin {
    pub amount: Option<i64>,
    #[avro(rename = "assetId")]
    pub asset_id: Option<AvroBytes>,
    pub to: Option<AvroBytes>,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputContract {
    #[avro(rename = "balanceRoot")]
    pub balance_root: Option<AvroBytes>,
    #[avro(rename = "inputIndex")]
    pub input_index: Option<i64>,
    #[avro(rename = "stateRoot")]
    pub state_root: Option<AvroBytes>,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputContractCreated {
    #[avro(rename = "contractId")]
    pub contract_id: Option<AvroBytes>,
    #[avro(rename = "stateRoot")]
    pub state_root: Option<AvroBytes>,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputChange {
    pub amount: Option<i64>,
    #[avro(rename = "assetId")]
    pub asset_id: Option<AvroBytes>,
    pub to: Option<AvroBytes>,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct OutputVariable {
    pub amount: Option<i64>,
    #[avro(rename = "assetId")]
    pub asset_id: Option<AvroBytes>,
    pub to: Option<AvroBytes>,
}

#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, AvroSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Outputs {
    #[avro(rename = "coinOutputs")]
    pub coin_outputs: Option<Vec<OutputCoin>>,
    #[avro(rename = "contractOutputs")]
    pub contract_outputs: Option<Vec<OutputContract>>,
    #[avro(rename = "changeOutputs")]
    pub change_outputs: Option<Vec<OutputChange>>,
    #[avro(rename = "variableOutputs")]
    pub variable_outputs: Option<Vec<OutputVariable>>,
    #[avro(rename = "contractCreatedOutputs")]
    pub contract_created_outputs: Option<Vec<OutputContractCreated>>,
    #[avro(rename = "outputTypes")]
    pub output_types: Option<Vec<String>>,
}

impl OutputCoin {
    pub fn new(output: &outputs::OutputCoin) -> Self {
        Self {
            amount: Some(output.amount.0 as i64),
            asset_id: Some(output.asset_id.clone().into()),
            to: Some(output.to.clone().into()),
        }
    }
}

impl OutputContract {
    pub fn new(output: &outputs::OutputContract) -> Self {
        Self {
            balance_root: Some(output.balance_root.clone().into()),
            input_index: Some(output.input_index as i64),
            state_root: Some(output.state_root.clone().into()),
        }
    }
}

impl OutputContractCreated {
    pub fn new(output: &outputs::OutputContractCreated) -> Self {
        Self {
            contract_id: Some(output.contract_id.clone().into()),
            state_root: Some(output.state_root.clone().into()),
        }
    }
}

impl OutputChange {
    pub fn new(output: &outputs::OutputChange) -> Self {
        Self {
            amount: Some(output.amount.0 as i64),
            asset_id: Some(output.asset_id.clone().into()),
            to: Some(output.to.clone().into()),
        }
    }
}

impl OutputVariable {
    pub fn new(output: &outputs::OutputVariable) -> Self {
        Self {
            amount: Some(output.amount.0 as i64),
            asset_id: Some(output.asset_id.clone().into()),
            to: Some(output.to.clone().into()),
        }
    }
}

impl Outputs {
    pub fn new(outputs: &[outputs::Output]) -> Self {
        let mut coin_outputs = Vec::new();
        let mut contract_outputs = Vec::new();
        let mut change_outputs = Vec::new();
        let mut variable_outputs = Vec::new();
        let mut contract_created_outputs = Vec::new();
        let mut output_types = Vec::new();

        for output in outputs {
            match output {
                outputs::Output::Coin(coin) => {
                    coin_outputs.push(OutputCoin::new(coin));
                    output_types.push("coin".to_string());
                }
                outputs::Output::Contract(contract) => {
                    contract_outputs.push(OutputContract::new(contract));
                    output_types.push("contract".to_string());
                }
                outputs::Output::Change(change) => {
                    change_outputs.push(OutputChange::new(change));
                    output_types.push("change".to_string());
                }
                outputs::Output::Variable(variable) => {
                    variable_outputs.push(OutputVariable::new(variable));
                    output_types.push("variable".to_string());
                }
                outputs::Output::ContractCreated(created) => {
                    contract_created_outputs
                        .push(OutputContractCreated::new(created));
                    output_types.push("contract_created".to_string());
                }
            }
        }

        Self {
            coin_outputs: Some(coin_outputs),
            contract_outputs: Some(contract_outputs),
            change_outputs: Some(change_outputs),
            variable_outputs: Some(variable_outputs),
            contract_created_outputs: Some(contract_created_outputs),
            output_types: Some(output_types),
        }
    }
}

#[cfg(test)]
mod tests {
    use apache_avro::AvroSchema;
    use fuel_streams_domains::outputs::types::MockOutput;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::helpers::{write_schema_files, AvroParser};

    // Helper function to reduce code duplication in tests
    fn test_output_serialization(parser: AvroParser, avro_output: Outputs) {
        // Test JSON serialization/deserialization
        let ser = serde_json::to_vec(&avro_output).unwrap();
        let deser = serde_json::from_slice::<Outputs>(&ser).unwrap();
        assert_eq!(avro_output, deser);

        // Test Avro serialization/deserialization
        let mut avro_writer = parser.writer_with_schema::<Outputs>().unwrap();
        avro_writer.append(&avro_output).unwrap();
        let serialized = avro_writer.into_inner().unwrap();
        let deserialized = parser
            .reader_with_schema::<Outputs>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(avro_output, deserialized[0]);
    }

    #[test]
    fn test_avro_output_coin() {
        let parser = AvroParser::default();
        let output = MockOutput::coin(1000);
        let avro_output = Outputs::new(&[output]);
        test_output_serialization(parser, avro_output);
    }

    #[test]
    fn test_avro_output_contract() {
        let parser = AvroParser::default();
        let output = MockOutput::contract();
        let avro_output = Outputs::new(&[output]);
        test_output_serialization(parser, avro_output);
    }

    #[test]
    fn test_avro_output_change() {
        let parser = AvroParser::default();
        let output = MockOutput::change(2000);
        let avro_output = Outputs::new(&[output]);
        test_output_serialization(parser, avro_output);
    }

    #[test]
    fn test_avro_output_variable() {
        let parser = AvroParser::default();
        let output = MockOutput::variable(3000);
        let avro_output = Outputs::new(&[output]);
        test_output_serialization(parser, avro_output);
    }

    #[test]
    fn test_avro_output_contract_created() {
        let parser = AvroParser::default();
        let output = MockOutput::contract_created();
        let avro_output = Outputs::new(&[output]);
        test_output_serialization(parser, avro_output);
    }

    #[test]
    fn test_avro_outputs_all() {
        let parser = AvroParser::default();
        let outputs = MockOutput::all();
        let avro_output = Outputs::new(&outputs);
        test_output_serialization(parser, avro_output);
    }

    #[tokio::test]
    async fn write_output_schemas() {
        // Write schemas for all output types
        let schemas = [
            ("output_coin.json", OutputCoin::get_schema()),
            ("output_contract.json", OutputContract::get_schema()),
            ("output_change.json", OutputChange::get_schema()),
            ("output_variable.json", OutputVariable::get_schema()),
            (
                "output_contract_created.json",
                OutputContractCreated::get_schema(),
            ),
            ("outputs.json", Outputs::get_schema()),
        ];

        write_schema_files(&schemas).await;
    }
}
