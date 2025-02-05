use fuel_streams_core::types::*;
use fuel_streams_domains::SubjectPayload;
use fuel_web_utils::server::api::API_VERSION;

use crate::server::errors::WebsocketError;

pub async fn decode_and_respond(
    payload: SubscriptionPayload,
    (subject, data): (String, Vec<u8>),
) -> Result<ServerResponse, WebsocketError> {
    let subject_payload: SubjectPayload = payload.try_into()?;
    let subject_dyn = subject_payload.into_subject()?;
    let subject_id = subject_dyn.id();
    let data = MessagePayload::new(subject_id, &data)?;
    let response_message = StreamMessage {
        subject,
        ty: subject_id.to_string(),
        version: API_VERSION.to_string(),
        payload: data,
    };
    Ok(ServerResponse::Response(response_message))
}
