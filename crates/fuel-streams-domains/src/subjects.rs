use std::{str::FromStr, sync::Arc};

use fuel_streams_macros::subject::{FromJsonString, IntoSubject};
use fuel_streams_store::record::RecordEntity;
use thiserror::Error;

use crate::{
    blocks::*,
    inputs::*,
    outputs::*,
    receipts::*,
    transactions::*,
    utxos::*,
};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SubjectPayloadError {
    #[error("Unknown subject: {0}")]
    UnknownSubject(String),
    #[error(transparent)]
    ParseError(#[from] fuel_streams_macros::subject::SubjectError),
}

#[derive(Debug, Clone)]
pub enum Subjects {
    Block(BlocksSubject),
    InputsCoin(InputsCoinSubject),
    InputsContract(InputsContractSubject),
    InputsMessage(InputsMessageSubject),
    OutputsCoin(OutputsCoinSubject),
    OutputsContract(OutputsContractSubject),
    OutputsChange(OutputsChangeSubject),
    OutputsVariable(OutputsVariableSubject),
    OutputsContractCreated(OutputsContractCreatedSubject),
    ReceiptsCall(ReceiptsCallSubject),
    ReceiptsReturn(ReceiptsReturnSubject),
    ReceiptsReturnData(ReceiptsReturnDataSubject),
    ReceiptsPanic(ReceiptsPanicSubject),
    ReceiptsRevert(ReceiptsRevertSubject),
    ReceiptsLog(ReceiptsLogSubject),
    ReceiptsLogData(ReceiptsLogDataSubject),
    ReceiptsTransfer(ReceiptsTransferSubject),
    ReceiptsTransferOut(ReceiptsTransferOutSubject),
    ReceiptsScriptResult(ReceiptsScriptResultSubject),
    ReceiptsMessageOut(ReceiptsMessageOutSubject),
    ReceiptsMint(ReceiptsMintSubject),
    ReceiptsBurn(ReceiptsBurnSubject),
    Transactions(TransactionsSubject),
    Utxos(UtxosSubject),
}

impl From<Subjects> for Arc<dyn IntoSubject> {
    fn from(subject: Subjects) -> Self {
        match subject {
            Subjects::Block(s) => s.dyn_arc(),
            Subjects::InputsCoin(s) => s.dyn_arc(),
            Subjects::InputsContract(s) => s.dyn_arc(),
            Subjects::InputsMessage(s) => s.dyn_arc(),
            Subjects::OutputsCoin(s) => s.dyn_arc(),
            Subjects::OutputsContract(s) => s.dyn_arc(),
            Subjects::OutputsChange(s) => s.dyn_arc(),
            Subjects::OutputsVariable(s) => s.dyn_arc(),
            Subjects::OutputsContractCreated(s) => s.dyn_arc(),
            Subjects::ReceiptsCall(s) => s.dyn_arc(),
            Subjects::ReceiptsReturn(s) => s.dyn_arc(),
            Subjects::ReceiptsReturnData(s) => s.dyn_arc(),
            Subjects::ReceiptsPanic(s) => s.dyn_arc(),
            Subjects::ReceiptsRevert(s) => s.dyn_arc(),
            Subjects::ReceiptsLog(s) => s.dyn_arc(),
            Subjects::ReceiptsLogData(s) => s.dyn_arc(),
            Subjects::ReceiptsTransfer(s) => s.dyn_arc(),
            Subjects::ReceiptsTransferOut(s) => s.dyn_arc(),
            Subjects::ReceiptsScriptResult(s) => s.dyn_arc(),
            Subjects::ReceiptsMessageOut(s) => s.dyn_arc(),
            Subjects::ReceiptsMint(s) => s.dyn_arc(),
            Subjects::ReceiptsBurn(s) => s.dyn_arc(),
            Subjects::Transactions(s) => s.dyn_arc(),
            Subjects::Utxos(s) => s.dyn_arc(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubjectPayload {
    pub subject: String,
    pub params: serde_json::Value,
    record_entity: RecordEntity,
}
impl SubjectPayload {
    pub fn new(
        subject: String,
        params: serde_json::Value,
    ) -> Result<Self, SubjectPayloadError> {
        let record_entity = Self::record_from_subject_str(&subject)?;
        Ok(Self {
            record_entity,
            subject,
            params,
        })
    }

    pub fn into_subject(&self) -> Arc<dyn IntoSubject> {
        let subject: Subjects = self.clone().try_into().unwrap();
        subject.into()
    }

    pub fn record_entity(&self) -> &RecordEntity {
        &self.record_entity
    }

    pub fn parsed_subject(&self) -> String {
        let subject_item = self.into_subject();
        subject_item.parse()
    }

    fn record_from_subject_str(
        subject: &str,
    ) -> Result<RecordEntity, SubjectPayloadError> {
        let subject = subject.to_lowercase();
        let subject_entity = if subject.contains("_") {
            subject.split("_").next().unwrap()
        } else {
            &subject
        };
        RecordEntity::from_str(subject_entity)
            .map_err(|_| SubjectPayloadError::UnknownSubject(subject))
    }
}

impl TryFrom<SubjectPayload> for Subjects {
    type Error = SubjectPayloadError;

    fn try_from(json: SubjectPayload) -> Result<Self, Self::Error> {
        match json.subject.as_str() {
            BlocksSubject::ID => Ok(Subjects::Block(BlocksSubject::from_json(
                &json.params.to_string(),
            )?)),
            InputsCoinSubject::ID => Ok(Subjects::InputsCoin(
                InputsCoinSubject::from_json(&json.params.to_string())?,
            )),
            InputsContractSubject::ID => Ok(Subjects::InputsContract(
                InputsContractSubject::from_json(&json.params.to_string())?,
            )),
            InputsMessageSubject::ID => Ok(Subjects::InputsMessage(
                InputsMessageSubject::from_json(&json.params.to_string())?,
            )),
            OutputsCoinSubject::ID => Ok(Subjects::OutputsCoin(
                OutputsCoinSubject::from_json(&json.params.to_string())?,
            )),
            OutputsContractSubject::ID => Ok(Subjects::OutputsContract(
                OutputsContractSubject::from_json(&json.params.to_string())?,
            )),
            OutputsChangeSubject::ID => Ok(Subjects::OutputsChange(
                OutputsChangeSubject::from_json(&json.params.to_string())?,
            )),
            OutputsVariableSubject::ID => Ok(Subjects::OutputsVariable(
                OutputsVariableSubject::from_json(&json.params.to_string())?,
            )),
            OutputsContractCreatedSubject::ID => {
                Ok(Subjects::OutputsContractCreated(
                    OutputsContractCreatedSubject::from_json(
                        &json.params.to_string(),
                    )?,
                ))
            }
            ReceiptsCallSubject::ID => Ok(Subjects::ReceiptsCall(
                ReceiptsCallSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsReturnSubject::ID => Ok(Subjects::ReceiptsReturn(
                ReceiptsReturnSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsReturnDataSubject::ID => Ok(Subjects::ReceiptsReturnData(
                ReceiptsReturnDataSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsPanicSubject::ID => Ok(Subjects::ReceiptsPanic(
                ReceiptsPanicSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsRevertSubject::ID => Ok(Subjects::ReceiptsRevert(
                ReceiptsRevertSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsLogSubject::ID => Ok(Subjects::ReceiptsLog(
                ReceiptsLogSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsLogDataSubject::ID => Ok(Subjects::ReceiptsLogData(
                ReceiptsLogDataSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsTransferSubject::ID => Ok(Subjects::ReceiptsTransfer(
                ReceiptsTransferSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsTransferOutSubject::ID => {
                Ok(Subjects::ReceiptsTransferOut(
                    ReceiptsTransferOutSubject::from_json(
                        &json.params.to_string(),
                    )?,
                ))
            }
            ReceiptsScriptResultSubject::ID => {
                Ok(Subjects::ReceiptsScriptResult(
                    ReceiptsScriptResultSubject::from_json(
                        &json.params.to_string(),
                    )?,
                ))
            }
            ReceiptsMessageOutSubject::ID => Ok(Subjects::ReceiptsMessageOut(
                ReceiptsMessageOutSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsMintSubject::ID => Ok(Subjects::ReceiptsMint(
                ReceiptsMintSubject::from_json(&json.params.to_string())?,
            )),
            ReceiptsBurnSubject::ID => Ok(Subjects::ReceiptsBurn(
                ReceiptsBurnSubject::from_json(&json.params.to_string())?,
            )),
            TransactionsSubject::ID => Ok(Subjects::Transactions(
                TransactionsSubject::from_json(&json.params.to_string())?,
            )),
            UtxosSubject::ID => Ok(Subjects::Utxos(UtxosSubject::from_json(
                &json.params.to_string(),
            )?)),
            _ => Err(SubjectPayloadError::UnknownSubject(json.subject)),
        }
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_store::record::RecordEntity;
    use serde_json::json;
    use test_case::test_case;

    use super::*;

    #[test]
    fn test_subject_json_conversion() {
        // Test block subject
        let block_json = SubjectPayload::new(
            BlocksSubject::ID.to_string(),
            json!({
                "producer": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "height": 123
            }),
        ).unwrap();
        let subject: Subjects = block_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::Block(_)));

        // Test inputs_coin subject
        let inputs_coin_json = SubjectPayload::new(
            InputsCoinSubject::ID.to_string(),
            json!({
                "block_height": 123,
                "tx_id": "0x0202020202020202020202020202020202020202020202020202020202020202",
                "tx_index": 0,
                "input_index": 1,
                "owner": "0x0303030303030303030303030303030303030303030303030303030303030303",
                "asset_id": "0x0404040404040404040404040404040404040404040404040404040404040404"
            }),
        ).unwrap();
        let subject: Subjects = inputs_coin_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::InputsCoin(_)));

        // Test with empty params
        let empty_block_json =
            SubjectPayload::new(BlocksSubject::ID.to_string(), json!({}))
                .unwrap();
        let subject: Subjects = empty_block_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::Block(_)));

        // Test invalid subject
        let result =
            SubjectPayload::new("invalid_subject".to_string(), json!({}));
        assert!(matches!(
            result,
            Err(SubjectPayloadError::UnknownSubject(_))
        ));
    }

    #[test_case("blocks" => Ok(RecordEntity::Block); "blocks subject")]
    #[test_case("inputs_coin" => Ok(RecordEntity::Input); "inputs_coin subject")]
    #[test_case("inputs_contract" => Ok(RecordEntity::Input); "inputs_contract subject")]
    #[test_case("inputs_message" => Ok(RecordEntity::Input); "inputs_message subject")]
    #[test_case("outputs_coin" => Ok(RecordEntity::Output); "outputs_coin subject")]
    #[test_case("outputs_contract" => Ok(RecordEntity::Output); "outputs_contract subject")]
    #[test_case("outputs_change" => Ok(RecordEntity::Output); "outputs_change subject")]
    #[test_case("outputs_variable" => Ok(RecordEntity::Output); "outputs_variable subject")]
    #[test_case("outputs_contract_created" => Ok(RecordEntity::Output); "outputs_contract_created subject")]
    #[test_case("receipts_call" => Ok(RecordEntity::Receipt); "receipts_call subject")]
    #[test_case("receipts_return" => Ok(RecordEntity::Receipt); "receipts_return subject")]
    #[test_case("receipts_return_data" => Ok(RecordEntity::Receipt); "receipts_return_data subject")]
    #[test_case("receipts_panic" => Ok(RecordEntity::Receipt); "receipts_panic subject")]
    #[test_case("receipts_revert" => Ok(RecordEntity::Receipt); "receipts_revert subject")]
    #[test_case("receipts_log" => Ok(RecordEntity::Receipt); "receipts_log subject")]
    #[test_case("receipts_log_data" => Ok(RecordEntity::Receipt); "receipts_log_data subject")]
    #[test_case("receipts_transfer" => Ok(RecordEntity::Receipt); "receipts_transfer subject")]
    #[test_case("receipts_transfer_out" => Ok(RecordEntity::Receipt); "receipts_transfer_out subject")]
    #[test_case("receipts_script_result" => Ok(RecordEntity::Receipt); "receipts_script_result subject")]
    #[test_case("receipts_message_out" => Ok(RecordEntity::Receipt); "receipts_message_out subject")]
    #[test_case("receipts_mint" => Ok(RecordEntity::Receipt); "receipts_mint subject")]
    #[test_case("receipts_burn" => Ok(RecordEntity::Receipt); "receipts_burn subject")]
    #[test_case("transactions" => Ok(RecordEntity::Transaction); "transactions subject")]
    #[test_case("utxos" => Ok(RecordEntity::Utxo); "utxos subject")]
    // Case variations
    #[test_case("BLOCKS" => Ok(RecordEntity::Block); "uppercase subject")]
    #[test_case("Inputs_Coin" => Ok(RecordEntity::Input); "mixed case subject")]
    #[test_case("RECEIPTS_TRANSFER" => Ok(RecordEntity::Receipt); "uppercase with underscore")]
    // Invalid cases
    #[test_case("invalid" => Err(SubjectPayloadError::UnknownSubject("invalid".to_string())); "invalid subject")]
    #[test_case("invalid_subject" => Err(SubjectPayloadError::UnknownSubject("invalid_subject".to_string())); "invalid subject with type")]
    #[test_case("" => Err(SubjectPayloadError::UnknownSubject("".to_string())); "empty subject")]
    fn test_record_entity_parsing(
        input: &str,
    ) -> Result<RecordEntity, SubjectPayloadError> {
        SubjectPayload::record_from_subject_str(input)
    }
}
