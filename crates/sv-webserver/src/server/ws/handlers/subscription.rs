use std::sync::Arc;

use actix_ws::Session;
use fuel_streams_core::{BoxedStream, DeliverPolicy, FuelStreams};
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::RecordEntity;
use fuel_web_utils::telemetry::Telemetry;
use futures::StreamExt;

use crate::{
    handle_ws_error,
    metrics::Metrics,
    server::ws::{
        context::WsContext,
        decoder::decode_record,
        errors::WsSubscriptionError,
        models::{ServerMessage, SubscriptionPayload},
    },
};

pub async fn handle_subscribe(
    payload: SubscriptionPayload,
    ctx: WsContext,
    session: Session,
    telemetry: &Arc<Telemetry<Metrics>>,
    streams: &Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received subscribe message: {:?}", payload);

    let mut ctx = ctx.with_payload(&payload);
    let subject_payload: SubjectPayload = payload.clone().try_into()?;
    let sub =
        create_subscriber(streams, &subject_payload, payload.deliver_policy)
            .await;

    let sub = handle_ws_error!(sub, ctx.clone());
    let stream_session = session.clone();
    ctx.send_message_to_socket(ServerMessage::Subscribed(payload.clone()))
        .await;

    // Start subscription processing in a background task
    actix_web::rt::spawn({
        let telemetry = telemetry.clone();
        async move {
            let _ = process_subscription(
                sub,
                stream_session,
                ctx,
                &telemetry,
                payload,
            )
            .await;
        }
    });

    Ok(())
}

pub async fn handle_unsubscribe(
    ctx: WsContext,
    payload: SubscriptionPayload,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received unsubscribe message: {:?}", payload);
    let mut ctx = ctx.with_payload(&payload);
    let msg = ServerMessage::Unsubscribed(payload.clone());
    if let Err(e) = serde_json::to_vec(&msg) {
        let error = WsSubscriptionError::UnserializablePayload(e);
        ctx.close_with_error(error).await;
        return Ok(());
    }

    ctx.send_message_to_socket(msg).await;
    Ok(())
}

async fn process_subscription(
    mut sub: BoxedStream,
    mut stream_session: Session,
    ctx: WsContext,
    telemetry: &Arc<Telemetry<Metrics>>,
    payload: SubscriptionPayload,
) -> Result<(), WsSubscriptionError> {
    let mut ctx = ctx.with_payload(&payload);
    let payload_str = payload.clone().to_string();
    let cleanup = || {
        if let Some(metrics) = telemetry.base_metrics() {
            metrics.update_unsubscribed(ctx.user_id, &payload_str);
            metrics.decrement_subscriptions_count();
        }
    };

    if let Some(metrics) = telemetry.base_metrics() {
        metrics.update_user_subscription_metrics(ctx.user_id, &payload_str);
    }

    while let Some(result) = sub.next().await {
        let result = result?;
        let payload = match decode_record(payload.to_owned(), result).await {
            Ok(res) => res,
            Err(e) => {
                if let Some(metrics) = telemetry.base_metrics() {
                    metrics.update_error_metrics(&payload_str, &e.to_string());
                }
                tracing::error!(
                    "Error serializing received stream message: {:?}",
                    e
                );
                continue;
            }
        };

        if let Err(e) = stream_session.binary(payload).await {
            tracing::error!("Error sending message over websocket: {:?}", e);
            cleanup();
            return Err(e.into());
        }
    }

    cleanup();

    let msg = ServerMessage::Unsubscribed(payload.clone());
    ctx.send_message_to_socket(msg).await;
    Ok(())
}

async fn create_subscriber(
    streams: &Arc<FuelStreams>,
    subject_json: &SubjectPayload,
    deliver_policy: DeliverPolicy,
) -> Result<BoxedStream, WsSubscriptionError> {
    let record_entity = subject_json.record_entity();
    let stream = match record_entity {
        RecordEntity::Block => {
            let subject = subject_json.into_subject();
            streams
                .blocks
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Transaction => {
            let subject = subject_json.into_subject();
            streams
                .transactions
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Input => {
            let subject = subject_json.into_subject();
            streams
                .inputs
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Output => {
            let subject = subject_json.into_subject();
            streams
                .outputs
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Receipt => {
            let subject = subject_json.into_subject();
            streams
                .receipts
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Utxo => {
            let subject = subject_json.into_subject();
            streams
                .utxos
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
    };
    Ok(Box::new(stream))
}
