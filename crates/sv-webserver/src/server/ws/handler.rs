use std::{str::FromStr, sync::Arc, time::Duration};

use actix_web::web::Bytes;
use actix_ws::Session;
use fuel_streams_core::FuelStreams;
use fuel_streams_store::record::RecordEntity;
use futures::StreamExt;
use tokio::time::sleep;
use uuid::Uuid;

use super::{
    context::WsContext,
    decoder::decode_record,
    errors::WsSubscriptionError,
    models::{
        ClientMessage,
        DeliverPolicy,
        ServerMessage,
        SubscriptionPayload,
    },
    socket::{send_message_to_socket, verify_and_extract_subject_name},
    subscriber::{create_subscriber, BoxedStream},
};
use crate::telemetry::Telemetry;

/// Macro to handle WebSocket errors using WsContext
macro_rules! handle_ws_error {
    ($result:expr, $ctx:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                $ctx.close_with_error(e.into()).await;
                return Ok(());
            }
        }
    };
}

/// Handles incoming binary messages from the WebSocket
pub async fn handle_binary_message(
    msg: Bytes,
    user_id: Uuid,
    session: Session,
    telemetry: Arc<Telemetry>,
    streams: Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received binary {:?}", msg);

    let ctx = WsContext::new(user_id, session.clone(), telemetry.clone());
    let client_message = handle_ws_error!(parse_client_message(msg), ctx);

    match client_message {
        ClientMessage::Subscribe(payload) => {
            handle_subscribe(payload, ctx, session, telemetry, streams).await
        }
        ClientMessage::Unsubscribe(payload) => {
            handle_unsubscribe(payload, ctx, session).await
        }
    }
}

/// Handles subscription requests
async fn handle_subscribe(
    payload: SubscriptionPayload,
    ctx: WsContext,
    mut session: Session,
    telemetry: Arc<Telemetry>,
    streams: Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received subscribe message: {:?}", payload);
    let subject_wildcard = payload.wildcard.clone();
    let deliver_policy = payload.deliver_policy;

    // verify the subject name
    let entity = handle_ws_error!(
        verify_and_extract_subject_name(&subject_wildcard),
        ctx.clone().with_subject_wildcard(&subject_wildcard)
    );

    let record_entity = RecordEntity::from_str(&entity)
        .map_err(|_| WsSubscriptionError::InvalidRecordEntity(entity.clone()));
    let record_entity = handle_ws_error!(
        record_entity,
        ctx.clone().with_subject_wildcard(&subject_wildcard)
    );

    let nats_client = streams.nats_client();
    let db = streams.db.clone();
    let sub = create_subscriber(
        &record_entity,
        &nats_client,
        &db,
        subject_wildcard.clone(),
        deliver_policy,
    )
    .await
    .map_err(WsSubscriptionError::StreamError);
    let sub = handle_ws_error!(
        sub,
        ctx.clone().with_subject_wildcard(&subject_wildcard)
    );

    let stream_session = session.clone();
    send_message_to_socket(
        &mut session,
        ServerMessage::Subscribed(SubscriptionPayload {
            wildcard: subject_wildcard.clone(),
            deliver_policy,
        }),
    )
    .await;

    // Start subscription processing in a background task
    actix_web::rt::spawn(async move {
        process_subscription(
            sub,
            stream_session,
            ctx.user_id,
            telemetry,
            subject_wildcard,
            record_entity,
        )
        .await;
    });

    Ok(())
}

/// Handles unsubscribe requests
async fn handle_unsubscribe(
    payload: SubscriptionPayload,
    ctx: WsContext,
    mut session: Session,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received unsubscribe message: {:?}", payload);
    let subject_wildcard = payload.wildcard.clone();
    let deliver_policy = payload.deliver_policy;

    let ctx = ctx.with_subject_wildcard(&subject_wildcard);

    // Verify the subject name
    handle_ws_error!(
        verify_and_extract_subject_name(&subject_wildcard),
        ctx.clone()
    );

    // Update metrics
    ctx.telemetry
        .update_unsubscribed(ctx.user_id, &subject_wildcard);
    ctx.telemetry.decrement_subscriptions_count();

    // Send unsubscribe confirmation
    let msg = ServerMessage::Unsubscribed(SubscriptionPayload {
        wildcard: subject_wildcard,
        deliver_policy,
    });

    if let Err(e) = serde_json::to_vec(&msg) {
        let error = WsSubscriptionError::UnserializablePayload(e);
        ctx.close_with_error(error).await;
        return Ok(());
    }

    send_message_to_socket(&mut session, msg).await;
    Ok(())
}

/// Parses a binary message into a ClientMessage
fn parse_client_message(
    msg: Bytes,
) -> Result<ClientMessage, WsSubscriptionError> {
    serde_json::from_slice(&msg)
        .map_err(WsSubscriptionError::UnserializablePayload)
}

/// Processes a subscription stream
async fn process_subscription(
    mut sub: BoxedStream,
    mut stream_session: Session,
    user_id: Uuid,
    telemetry: Arc<Telemetry>,
    subject_wildcard: String,
    record_entity: RecordEntity,
) {
    telemetry.update_user_subscription_metrics(user_id, &subject_wildcard);

    let cleanup = || {
        telemetry.update_unsubscribed(user_id, &subject_wildcard);
        telemetry.decrement_subscriptions_count();
    };

    while let Some(result) = sub.next().await {
        let serialized_ws_payload = match decode_record(&record_entity, result)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                telemetry
                    .update_error_metrics(&subject_wildcard, &e.to_string());
                tracing::error!(
                    "Error serializing received stream message: {:?}",
                    e
                );

                // Send error message to client
                if let Ok(error_msg) =
                    serde_json::to_vec(&ServerMessage::Error(e.to_string()))
                {
                    if stream_session.binary(error_msg).await.is_err() {
                        tracing::error!(
                            "Failed to send error message to client"
                        );
                        cleanup();
                        return;
                    }
                }
                continue;
            }
        };

        if let Err(e) = stream_session.binary(serialized_ws_payload).await {
            tracing::error!("Error sending message over websocket: {:?}", e);
            cleanup();
            return;
        }

        // Add delay to throttle data streaming
        sleep(Duration::from_millis(500)).await;
    }

    // Stream ended normally
    cleanup();

    // Send unsubscribe message to client
    let _ = send_message_to_socket(
        &mut stream_session,
        ServerMessage::Unsubscribed(SubscriptionPayload {
            wildcard: subject_wildcard,
            deliver_policy: DeliverPolicy::All, // Default value since we don't have the original
        }),
    )
    .await;
}
