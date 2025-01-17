use std::sync::Arc;

use actix_ws::{CloseReason, Session};
use fuel_streams_core::{
    server::{DeliverPolicy, ServerMessage, Subscription},
    BoxedStream,
    FuelStreams,
};
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::RecordEntity;
use futures::StreamExt;

use super::decoder::decode_and_respond;
use crate::server::{errors::WebsocketError, websocket::WsSession};

pub async fn unsubscribe(
    session: &mut Session,
    ctx: &mut WsSession,
    subscription: &Subscription,
) -> Result<(), WebsocketError> {
    ctx.remove_subscription(subscription).await;
    let payload = subscription.payload();
    let msg = ServerMessage::Unsubscribed(payload.clone());
    ctx.send_message(session, msg).await?;
    Ok(())
}

pub async fn subscribe(
    session: &mut Session,
    ctx: &mut WsSession,
    subscription: &Subscription,
) -> Result<(), WebsocketError> {
    let payload = subscription.payload();
    tracing::info!("Received subscribe message: {:?}", payload);
    if ctx
        .check_duplicate_subscription(session, subscription)
        .await?
    {
        return Ok(());
    }

    // Subscribe to the subject
    let subject_payload: SubjectPayload = payload.clone().try_into()?;
    let sub = create_subscriber(
        &ctx.streams,
        &subject_payload,
        payload.deliver_policy,
    )
    .await?;

    // Send the subscription message to the client
    let subscribed_msg = ServerMessage::Subscribed(payload.clone());
    ctx.send_message(session, subscribed_msg).await?;
    ctx.add_subscription(subscription).await?;

    // Spawn a task to process messages
    spawn_subscription_process(session, ctx, subscription, sub);
    Ok(())
}

fn spawn_subscription_process(
    session: &mut Session,
    ctx: &mut WsSession,
    subscription: &Subscription,
    mut sub: BoxedStream,
) -> tokio::task::JoinHandle<()> {
    let api_key = ctx.api_key();
    let mut shutdown_rx = ctx.receiver();
    let payload = subscription.payload();
    tracing::debug!(%api_key, ?payload, "Starting subscription process");
    actix_web::rt::spawn({
        let mut session = session.to_owned();
        let mut ctx = ctx.to_owned();
        let subscription = subscription.clone();
        async move {
            let result: Result<(), WebsocketError> = tokio::select! {
                shutdown = shutdown_rx.recv() => {
                    match shutdown {
                        Ok(shutdown_api_key) if shutdown_api_key == api_key => {
                            tracing::info!(%api_key, "Subscription gracefully shutdown");
                            Ok(())
                        }
                        other => {
                            tracing::debug!(%api_key, ?other, "Unexpected shutdown value, falling back to process_msgs");
                            process_msgs(&mut session, &mut ctx, &mut sub, &subscription).await
                        }
                    }
                },
                process_result = process_msgs(&mut session, &mut ctx, &mut sub, &subscription) => {
                    tracing::debug!(%api_key, ?process_result, "Process messages completed");
                    process_result
                }
            };

            if let Err(err) = result {
                tracing::error!(%api_key, error = %err, "Subscription processing error");
                let _ = session.close(Some(CloseReason::from(err))).await;
            }
        }
    })
}

async fn process_msgs(
    session: &mut Session,
    ctx: &mut WsSession,
    sub: &mut BoxedStream,
    subscription: &Subscription,
) -> Result<(), WebsocketError> {
    let payload = subscription.payload();
    tracing::debug!(?payload, "Starting to process messages");
    while let Some(result) = sub.next().await {
        tracing::debug!(?payload, ?result, "Received message from stream");
        let result = result?;
        let payload = decode_and_respond(payload.to_owned(), result).await?;
        tracing::debug!("Sending message to client: {:?}", payload);
        ctx.send_message(session, payload).await?;
    }

    tracing::debug!(?payload, "Stream ended, removing subscription");
    ctx.remove_subscription(subscription).await;
    let msg = ServerMessage::Unsubscribed(payload.clone());
    ctx.send_message(session, msg).await?;
    Ok(())
}

async fn create_subscriber(
    streams: &Arc<FuelStreams>,
    subject_payload: &SubjectPayload,
    deliver_policy: DeliverPolicy,
) -> Result<BoxedStream, WebsocketError> {
    let subject = subject_payload.into_subject();
    let stream = match subject_payload.record_entity() {
        RecordEntity::Block => {
            streams
                .blocks
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Transaction => {
            streams
                .transactions
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Input => {
            streams
                .inputs
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Output => {
            streams
                .outputs
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Receipt => {
            streams
                .receipts
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
        RecordEntity::Utxo => {
            streams
                .utxos
                .subscribe_dynamic(subject, deliver_policy)
                .await
        }
    };
    Ok(Box::new(stream))
}
