use std::{str::FromStr, sync::Arc};

use fuel_streams_macros::subject::IntoSubject;
use fuel_streams_store::record::RecordEntity;
use thiserror::Error;

use crate::Subjects;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SubjectPayloadError {
    #[error("Unknown subject: {0}")]
    UnknownSubject(String),
    #[error(transparent)]
    ParseError(#[from] fuel_streams_macros::subject::SubjectError),
}

#[derive(Debug, Clone)]
pub struct SubjectPayload {
    pub subject: String,
    pub params: String,
    record_entity: RecordEntity,
}
impl SubjectPayload {
    pub fn new(
        subject: String,
        params: String,
    ) -> Result<Self, SubjectPayloadError> {
        let record_entity = Self::record_from_subject_str(&subject)?;
        Ok(Self {
            record_entity,
            subject,
            params,
        })
    }

    pub fn into_subject(
        &self,
    ) -> Result<Arc<dyn IntoSubject>, SubjectPayloadError> {
        let subject: Subjects = self.clone().try_into()?;
        Ok(subject.into())
    }

    pub fn record_entity(&self) -> &RecordEntity {
        &self.record_entity
    }

    pub fn parsed_subject(&self) -> Result<String, SubjectPayloadError> {
        let subject_item = self.into_subject()?;
        Ok(subject_item.parse())
    }

    pub fn record_from_subject_str(
        subject: &str,
    ) -> Result<RecordEntity, SubjectPayloadError> {
        let subject = subject.to_lowercase();
        let subject_entity = if subject.contains("_") {
            subject
                .split("_")
                .next()
                .ok_or(SubjectPayloadError::UnknownSubject(subject.clone()))?
        } else {
            &subject
        };
        RecordEntity::from_str(subject_entity)
            .map_err(|_| SubjectPayloadError::UnknownSubject(subject))
    }
}

#[cfg(test)]
mod tests {
    use fuel_streams_store::record::RecordEntity;
    use serde_json::json;
    use test_case::test_case;

    use super::*;
    use crate::{blocks::BlocksSubject, inputs::InputsCoinSubject};

    #[test]
    fn test_subject_json_conversion() {
        // Test block subject
        let block_json = SubjectPayload::new(
            BlocksSubject::ID.to_string(),
            json!({
                "producer": "0x0101010101010101010101010101010101010101010101010101010101010101",
                "height": 123
            }).to_string(),
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
            }).to_string(),
        ).unwrap();
        let subject: Subjects = inputs_coin_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::InputsCoin(_)));

        // Test with empty params
        let empty_block_json = SubjectPayload::new(
            BlocksSubject::ID.to_string(),
            json!({}).to_string(),
        )
        .unwrap();
        let subject: Subjects = empty_block_json.try_into().unwrap();
        assert!(matches!(subject, Subjects::Block(_)));

        // Test invalid subject
        let result = SubjectPayload::new(
            "invalid_subject".to_string(),
            json!({}).to_string(),
        );
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
