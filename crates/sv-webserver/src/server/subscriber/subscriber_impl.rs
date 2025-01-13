use std::sync::Arc;

use fuel_streams_core::{BoxedStream, DeliverPolicy, FuelStreams};
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::RecordEntity;
use futures::StreamExt;

use super::decoder::decode_and_responde;
use crate::{
    handle_ws_error,
    server::{
        errors::WebsocketError,
        types::{ServerMessage, SubscriptionPayload},
        ws_context::WsContext,
    },
};

pub async fn subscribe(
    payload: SubscriptionPayload,
    ctx: WsContext,
) -> Result<(), WebsocketError> {
    tracing::info!("Received subscribe message: {:?}", payload);

    let mut ctx = ctx.with_payload(&payload);
    let subject_payload: SubjectPayload = payload.clone().try_into()?;
    let sub = create_subscriber(
        &ctx.streams,
        &subject_payload,
        payload.deliver_policy,
    )
    .await;

    let sub = handle_ws_error!(sub, ctx.clone());
    ctx.send_message_to_socket(ServerMessage::Subscribed(payload.clone()))
        .await;

    actix_web::rt::spawn({
        async move {
            let _ = process_subscription(sub, ctx, payload).await;
        }
    });
    Ok(())
}

pub async fn unsubscribe(
    ctx: WsContext,
    payload: SubscriptionPayload,
) -> Result<(), WebsocketError> {
    tracing::info!("Received unsubscribe message: {:?}", payload);
    let mut ctx = ctx.with_payload(&payload);
    let msg = ServerMessage::Unsubscribed(payload.clone());
    if let Err(e) = serde_json::to_vec(&msg) {
        let error = WebsocketError::UnserializablePayload(e);
        ctx.close_with_error(error).await;
        return Ok(());
    }

    ctx.send_message_to_socket(msg).await;
    Ok(())
}

async fn process_subscription(
    mut sub: BoxedStream,
    ctx: WsContext,
    payload: SubscriptionPayload,
) -> Result<(), WebsocketError> {
    let mut ctx = ctx.with_payload(&payload);
    let payload_str = payload.clone().to_string();
    let cleanup = || {
        if let Some(metrics) = ctx.telemetry.base_metrics() {
            metrics.update_unsubscribed(ctx.user_id, &payload_str);
            metrics.decrement_subscriptions_count();
        }
    };

    if let Some(metrics) = ctx.telemetry.base_metrics() {
        metrics.update_user_subscription_metrics(ctx.user_id, &payload_str);
    }

    while let Some(result) = sub.next().await {
        let result = result?;
        let payload = match decode_and_responde(payload.to_owned(), result)
            .await
        {
            Ok(res) => res,
            Err(e) => {
                if let Some(metrics) = ctx.telemetry.base_metrics() {
                    metrics.update_error_metrics(&payload_str, &e.to_string());
                }
                tracing::error!(
                    "Error serializing received stream message: {:?}",
                    e
                );
                continue;
            }
        };

        if let Err(e) = ctx.session.binary(payload).await {
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
) -> Result<BoxedStream, WebsocketError> {
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
