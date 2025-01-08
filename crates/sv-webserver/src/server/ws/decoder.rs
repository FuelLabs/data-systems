use fuel_streams_core::types::*;
use fuel_streams_store::{
    db::DbRecord,
    record::{DataEncoder, RecordEntity},
    store::StoreResult,
};

use super::{
    errors::WsSubscriptionError,
    models::{ResponseMessage, ServerMessage},
    socket::verify_and_extract_subject_name,
};

/// Decodes a record based on its type and converts it to a WebSocket message
pub async fn decode_record(
    stream_type: &RecordEntity,
    result: StoreResult<DbRecord>,
) -> Result<Vec<u8>, WsSubscriptionError> {
    let record = result?;
    let subject = record.subject.clone();
    let subject = verify_and_extract_subject_name(&subject)?;
    let json_value = decode_to_json_value(stream_type, record).await?;

    serde_json::to_vec(&ServerMessage::Response(ResponseMessage {
        subject: subject.to_string(),
        payload: json_value,
    }))
    .map_err(WsSubscriptionError::UnserializablePayload)
}

/// Decodes a record to its JSON value based on its type
async fn decode_to_json_value(
    stream_type: &RecordEntity,
    record: DbRecord,
) -> Result<serde_json::Value, WsSubscriptionError> {
    let value = match stream_type {
        RecordEntity::Block => {
            let payload: Block = record.decode_to_record().await?;
            payload.to_json_value()?
        }
        RecordEntity::Transaction => {
            let payload: Transaction = record.decode_to_record().await?;
            payload.to_json_value()?
        }
        RecordEntity::Input => {
            let payload: Input = record.decode_to_record().await?;
            payload.to_json_value()?
        }
        RecordEntity::Output => {
            let payload: Output = record.decode_to_record().await?;
            payload.to_json_value()?
        }
        RecordEntity::Receipt => {
            let payload: Receipt = record.decode_to_record().await?;
            payload.to_json_value()?
        }
        RecordEntity::Utxo => {
            let payload: Utxo = record.decode_to_record().await?;
            payload.to_json_value()?
        }
        RecordEntity::Log => {
            let payload: Log = record.decode_to_record().await?;
            payload.to_json_value()?
        }
    };
    Ok(value)
}
