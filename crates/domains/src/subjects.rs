use std::sync::Arc;

use fuel_streams_subject::subject::{
    IntoSubject,
    SubjectPayload,
    SubjectPayloadError,
};

use crate::{
    blocks::*,
    inputs::*,
    messages::MessagesSubject,
    outputs::*,
    predicates::*,
    receipts::*,
    transactions::*,
    utxos::*,
};

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum SubjectsError {
    #[error("Unknown subject when converting to Subjects: {0}")]
    UnknownSubject(String),
    #[error(transparent)]
    SubjectPayload(#[from] SubjectPayloadError),
}

#[derive(Debug, Clone)]
pub enum Subjects {
    Block(BlocksSubject),
    Inputs(InputsSubject),
    InputsCoin(InputsCoinSubject),
    InputsContract(InputsContractSubject),
    InputsMessage(InputsMessageSubject),
    Outputs(OutputsSubject),
    OutputsCoin(OutputsCoinSubject),
    OutputsContract(OutputsContractSubject),
    OutputsChange(OutputsChangeSubject),
    OutputsVariable(OutputsVariableSubject),
    OutputsContractCreated(OutputsContractCreatedSubject),
    Predicates(PredicatesSubject),
    Receipts(ReceiptsSubject),
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
    Messages(MessagesSubject),
}

impl From<Subjects> for Arc<dyn IntoSubject> {
    fn from(subject: Subjects) -> Self {
        match subject {
            Subjects::Block(s) => s.dyn_arc(),
            Subjects::Inputs(s) => s.dyn_arc(),
            Subjects::InputsCoin(s) => s.dyn_arc(),
            Subjects::InputsContract(s) => s.dyn_arc(),
            Subjects::InputsMessage(s) => s.dyn_arc(),
            Subjects::Outputs(s) => s.dyn_arc(),
            Subjects::OutputsCoin(s) => s.dyn_arc(),
            Subjects::OutputsContract(s) => s.dyn_arc(),
            Subjects::OutputsChange(s) => s.dyn_arc(),
            Subjects::OutputsVariable(s) => s.dyn_arc(),
            Subjects::OutputsContractCreated(s) => s.dyn_arc(),
            Subjects::Predicates(s) => s.dyn_arc(),
            Subjects::Receipts(s) => s.dyn_arc(),
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
            Subjects::Messages(s) => s.dyn_arc(),
        }
    }
}

macro_rules! impl_try_from_subjects {
    ($(($subject_type:ty, $variant:ident)),+ $(,)?) => {
        // Implementation for SubjectPayload
        impl TryFrom<SubjectPayload> for Subjects {
            type Error = SubjectsError;
            fn try_from(payload: SubjectPayload) -> Result<Self, Self::Error> {
                match payload.subject.as_str() {
                    $(<$subject_type>::ID => {
                        Ok(Subjects::$variant(payload.try_into()?))
                    },)+
                    _ => Err(SubjectsError::UnknownSubject(payload.subject))
                }
            }
        }
    };
}

impl_try_from_subjects!(
    // Block subjects
    (BlocksSubject, Block),
    // Input subjects
    (InputsSubject, Inputs),
    (InputsCoinSubject, InputsCoin),
    (InputsContractSubject, InputsContract),
    (InputsMessageSubject, InputsMessage),
    // Output subjects
    (OutputsSubject, Outputs),
    (OutputsCoinSubject, OutputsCoin),
    (OutputsContractSubject, OutputsContract),
    (OutputsChangeSubject, OutputsChange),
    (OutputsVariableSubject, OutputsVariable),
    (OutputsContractCreatedSubject, OutputsContractCreated),
    // Receipt subjects
    (ReceiptsSubject, Receipts),
    (ReceiptsCallSubject, ReceiptsCall),
    (ReceiptsReturnSubject, ReceiptsReturn),
    (ReceiptsReturnDataSubject, ReceiptsReturnData),
    (ReceiptsPanicSubject, ReceiptsPanic),
    (ReceiptsRevertSubject, ReceiptsRevert),
    (ReceiptsLogSubject, ReceiptsLog),
    (ReceiptsLogDataSubject, ReceiptsLogData),
    (ReceiptsTransferSubject, ReceiptsTransfer),
    (ReceiptsTransferOutSubject, ReceiptsTransferOut),
    (ReceiptsScriptResultSubject, ReceiptsScriptResult),
    (ReceiptsMessageOutSubject, ReceiptsMessageOut),
    (ReceiptsMintSubject, ReceiptsMint),
    (ReceiptsBurnSubject, ReceiptsBurn),
    // Transaction subjects
    (TransactionsSubject, Transactions),
    // Utxo subjects
    (UtxosSubject, Utxos),
    // Predicate transactions subject
    (PredicatesSubject, Predicates),
    // Message subjects
    (MessagesSubject, Messages),
);

#[allow(clippy::disallowed_macros)]
#[cfg(test)]
mod tests {
    use serde_json::json;
    use test_case::test_case;

    use super::*;
    use crate::infra::{RecordEntity, RecordEntityError};

    #[test]
    fn test_subject_json_conversion() {
        // Test block subject
        let block_json = SubjectPayload {
            subject: BlocksSubject::ID.to_string(),
            params: json!({
                "producer": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "height": 123,
                "da_height": 123
            }),
        };

        let subject: Subjects = block_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::Block(_)));

        // Test inputs_coin subject
        let inputs_coin_json = SubjectPayload {
            subject: InputsCoinSubject::ID.to_string(),
            params: json!({
                "block_height": 123,
                "tx_id": "0x0202020202020202020202020202020202020202020202020202020202020202",
                "tx_index": 0,
                "input_index": 1,
                "owner": "0x0303030303030303030303030303030303030303030303030303030303030303",
                "asset": "0x0404040404040404040404040404040404040404040404040404040404040404"
            }),
        };

        let subject: Subjects = inputs_coin_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::InputsCoin(_)));

        // Test with empty params
        let empty_block_json = SubjectPayload {
            subject: BlocksSubject::ID.to_string(),
            params: json!({}),
        };
        let subject: Subjects = empty_block_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::Block(_)));
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
    #[test_case("invalid" => Err(RecordEntityError::UnknownSubject("invalid".to_string())); "invalid subject")]
    #[test_case("invalid_subject" => Err(RecordEntityError::UnknownSubject("invalid_subject".to_string())); "invalid subject with type")]
    #[test_case("" => Err(RecordEntityError::UnknownSubject("".to_string())); "empty subject")]
    fn test_record_entity_parsing(
        input: &str,
    ) -> Result<RecordEntity, RecordEntityError> {
        RecordEntity::from_subject_id(input)
    }
}
