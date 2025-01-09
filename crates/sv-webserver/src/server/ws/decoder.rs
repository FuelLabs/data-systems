use fuel_streams_core::types::*;
use fuel_streams_store::record::{DataEncoder, RecordEntity};

use super::{
    errors::WsSubscriptionError,
    models::{ResponseMessage, ServerMessage},
    socket::verify_and_extract_subject_name,
};

/// Decodes a record based on its type and converts it to a WebSocket message
pub async fn decode_record(
    stream_type: &RecordEntity,
    (subject, payload): (String, Vec<u8>),
) -> Result<Vec<u8>, WsSubscriptionError> {
    let subject = verify_and_extract_subject_name(&subject)?;
    let json_value = decode_to_json_value(stream_type, payload).await?;
    serde_json::to_vec(&ServerMessage::Response(ResponseMessage {
        subject: subject.to_string(),
        payload: json_value,
    }))
    .map_err(WsSubscriptionError::UnserializablePayload)
}

/// Decodes a record to its JSON value based on its type
async fn decode_to_json_value(
    stream_type: &RecordEntity,
    payload: Vec<u8>,
) -> Result<serde_json::Value, WsSubscriptionError> {
    let value = match stream_type {
        RecordEntity::Block => {
            let payload: Block = Block::decode(&payload).await?;
            payload.to_json_value()?
        }
        RecordEntity::Transaction => {
            let payload: Transaction = Transaction::decode(&payload).await?;
            payload.to_json_value()?
        }
        RecordEntity::Input => {
            let payload: Input = Input::decode(&payload).await?;
            payload.to_json_value()?
        }
        RecordEntity::Output => {
            let payload: Output = Output::decode(&payload).await?;
            payload.to_json_value()?
        }
        RecordEntity::Receipt => {
            let payload: Receipt = Receipt::decode(&payload).await?;
            payload.to_json_value()?
        }
        RecordEntity::Utxo => {
            let payload: Utxo = Utxo::decode(&payload).await?;
            payload.to_json_value()?
        }
    };
    Ok(value)
}
