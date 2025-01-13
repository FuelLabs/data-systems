use std::sync::Arc;

use actix_ws::Session;
use bytes::Bytes;
use fuel_streams_core::FuelStreams;
use fuel_web_utils::telemetry::Telemetry;
use uuid::Uuid;

use crate::{
    handle_ws_error,
    metrics::Metrics,
    server::ws::{
        context::WsContext,
        errors::WsSubscriptionError,
        handlers::subscription::{handle_subscribe, handle_unsubscribe},
        models::ClientMessage,
    },
};

pub async fn handle_message(
    msg: Bytes,
    user_id: Uuid,
    session: Session,
    telemetry: &Arc<Telemetry<Metrics>>,
    streams: &Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received binary {:?}", msg);

    let ctx = WsContext::new(user_id, session.clone(), telemetry.clone());
    let parsed_message = serde_json::from_slice(&msg)
        .map_err(WsSubscriptionError::UnserializablePayload);
    let client_message = handle_ws_error!(parsed_message, ctx);

    match client_message {
        ClientMessage::Subscribe(payload) => {
            handle_subscribe(payload, ctx, session, telemetry, streams).await
        }
        ClientMessage::Unsubscribe(payload) => {
            handle_unsubscribe(ctx, payload).await
        }
    }
}
