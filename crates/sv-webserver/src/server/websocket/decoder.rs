use fuel_streams_core::{
    server::{ResponseMessage, ServerMessage, SubscriptionPayload},
    types::*,
};
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::{DataEncoder, RecordEntity};

use crate::server::errors::WebsocketError;

pub async fn decode_and_respond(
    payload: SubscriptionPayload,
    data: Vec<u8>,
) -> Result<ServerMessage, WebsocketError> {
    let subject = payload.subject.clone();
    let payload = decode_to_json_value(&payload.try_into()?, data).await?;
    let response_message = ResponseMessage { subject, payload };
    Ok(ServerMessage::Response(response_message))
}

async fn decode_to_json_value(
    payload: &SubjectPayload,
    data: Vec<u8>,
) -> Result<serde_json::Value, WebsocketError> {
    let value = match payload.record_entity() {
        RecordEntity::Block => {
            let payload: Block = Block::decode(&data).await?;
            payload.to_json_value()?
        }
        RecordEntity::Transaction => {
            let payload: Transaction = Transaction::decode(&data).await?;
            payload.to_json_value()?
        }
        RecordEntity::Input => {
            let payload: Input = Input::decode(&data).await?;
            payload.to_json_value()?
        }
        RecordEntity::Output => {
            let payload: Output = Output::decode(&data).await?;
            payload.to_json_value()?
        }
        RecordEntity::Receipt => {
            let payload: Receipt = Receipt::decode(&data).await?;
            payload.to_json_value()?
        }
        RecordEntity::Utxo => {
            let payload: Utxo = Utxo::decode(&data).await?;
            payload.to_json_value()?
        }
    };
    Ok(value)
}
