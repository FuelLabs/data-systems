use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use actix_web::web::Bytes;
use actix_ws::Session;
use fuel_streams_core::{BoxedStream, FuelStreams};
use fuel_streams_domains::SubjectPayload;
use futures::StreamExt;
use tokio::time::sleep;
use uuid::Uuid;

use super::{
    context::WsContext,
    decoder::decode_record,
    errors::WsSubscriptionError,
    models::{ClientMessage, ServerMessage, SubscriptionPayload},
    socket::send_message_to_socket,
};
use crate::{
    server::ws::{
        models::DeliverPolicy,
        subscriber::{create_historical_subscriber, create_live_subscriber},
    },
    telemetry::Telemetry,
};

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

pub async fn handle_binary_message(
    msg: Bytes,
    user_id: Uuid,
    session: Session,
    telemetry: &Arc<Telemetry>,
    streams: &Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received binary {:?}", msg);

    let ctx = WsContext::new(user_id, session.clone(), telemetry.clone());
    let client_message = handle_ws_error!(parse_client_message(msg), ctx);

    match client_message {
        ClientMessage::Subscribe(payload) => {
            handle_subscribe(payload, ctx, session, telemetry, streams).await
        }
        ClientMessage::Unsubscribe(payload) => {
            handle_unsubscribe(ctx, session, payload).await
        }
    }
}

async fn handle_subscribe(
    payload: SubscriptionPayload,
    ctx: WsContext,
    mut session: Session,
    telemetry: &Arc<Telemetry>,
    streams: &Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received subscribe message: {:?}", payload);

    let ctx = ctx.with_payload(&payload);
    let subject_payload: SubjectPayload = payload.clone().try_into()?;
    let sub = match payload.deliver_policy {
        DeliverPolicy::All => {
            create_historical_subscriber(streams, &subject_payload).await
        }
        _ => create_live_subscriber(streams, &subject_payload).await,
    };

    let sub = handle_ws_error!(sub, ctx.clone());
    let stream_session = session.clone();
    send_message_to_socket(
        &mut session,
        ServerMessage::Subscribed(payload.clone()),
    )
    .await;

    // Start subscription processing in a background task
    actix_web::rt::spawn({
        let telemetry = telemetry.clone();
        async move {
            process_subscription(
                sub,
                stream_session,
                ctx.user_id,
                &telemetry,
                payload,
            )
            .await;
        }
    });

    Ok(())
}

async fn handle_unsubscribe(
    ctx: WsContext,
    mut session: Session,
    payload: SubscriptionPayload,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received unsubscribe message: {:?}", payload);
    let ctx = ctx.with_payload(&payload);

    // Update metrics
    let payload_str = payload.to_string();
    ctx.telemetry.update_unsubscribed(ctx.user_id, &payload_str);
    ctx.telemetry.decrement_subscriptions_count();

    // Send unsubscribe confirmation
    let msg = ServerMessage::Unsubscribed(payload.clone());
    if let Err(e) = serde_json::to_vec(&msg) {
        let error = WsSubscriptionError::UnserializablePayload(e);
        ctx.close_with_error(error).await;
        return Ok(());
    }

    send_message_to_socket(&mut session, msg).await;
    Ok(())
}

fn parse_client_message(
    msg: Bytes,
) -> Result<ClientMessage, WsSubscriptionError> {
    serde_json::from_slice(&msg)
        .map_err(WsSubscriptionError::UnserializablePayload)
}

pub static WS_THROTTLE_TIME: LazyLock<usize> = LazyLock::new(|| {
    dotenvy::var("WS_THROTTLE_TIME")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(300)
});

async fn process_subscription(
    mut sub: BoxedStream,
    mut stream_session: Session,
    user_id: Uuid,
    telemetry: &Arc<Telemetry>,
    payload: SubscriptionPayload,
) {
    let payload_str = payload.clone().to_string();
    telemetry.update_user_subscription_metrics(user_id, &payload_str);
    let cleanup = || {
        telemetry.update_unsubscribed(user_id, &payload_str);
        telemetry.decrement_subscriptions_count();
    };

    while let Some(result) = sub.next().await {
        let payload = match decode_record(payload.to_owned(), result).await {
            Ok(res) => res,
            Err(e) => {
                telemetry.update_error_metrics(&payload_str, &e.to_string());
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

        if let Err(e) = stream_session.binary(payload).await {
            tracing::error!("Error sending message over websocket: {:?}", e);
            cleanup();
            return;
        }

        // Add delay to throttle data streaming
        sleep(Duration::from_millis(*WS_THROTTLE_TIME as u64)).await;
    }

    // Stream ended normally
    cleanup();

    // Send unsubscribe message to client
    let _ = send_message_to_socket(
        &mut stream_session,
        ServerMessage::Unsubscribed(payload.clone()),
    )
    .await;
}
