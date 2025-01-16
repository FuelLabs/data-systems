use std::sync::Arc;

use actix_ws::{CloseReason, Session};
use fuel_streams_core::{BoxedStream, DeliverPolicy, FuelStreams};
use fuel_streams_domains::SubjectPayload;
use fuel_streams_store::record::RecordEntity;
use futures::StreamExt;

use super::decoder::decode_and_response;
use crate::server::{
    errors::WebsocketError,
    types::{ServerMessage, SubscriptionPayload},
    websocket::WsController,
};

pub async fn unsubscribe(
    session: &mut Session,
    ctx: &mut WsController,
    payload: SubscriptionPayload,
) -> Result<(), WebsocketError> {
    ctx.remove_subscription(&payload).await;
    let msg = ServerMessage::Unsubscribed(payload.clone());
    ctx.send_message(session, msg).await?;
    Ok(())
}

pub async fn subscribe(
    session: &mut Session,
    ctx: &mut WsController,
    payload: SubscriptionPayload,
) -> Result<(), WebsocketError> {
    tracing::info!("Received subscribe message: {:?}", payload);
    if ctx.check_duplicate_subscription(session, &payload).await? {
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
    ctx.add_subscription(&payload).await?;

    // Spawn a task to process messages
    spawn_subscription_process(session, ctx, payload, sub);
    Ok(())
}

fn spawn_subscription_process(
    session: &mut Session,
    ctx: &mut WsController,
    payload: SubscriptionPayload,
    mut sub: BoxedStream,
) -> tokio::task::JoinHandle<()> {
    let user_id = *ctx.user_id();
    let mut shutdown_rx = ctx.receiver();
    actix_web::rt::spawn({
        let mut session = session.to_owned();
        let mut ctx = ctx.to_owned();
        async move {
            let result: Result<(), WebsocketError> = tokio::select! {
                shutdown = shutdown_rx.recv() => match shutdown {
                    Ok(shutdown_user_id) if shutdown_user_id == user_id => {
                        tracing::info!(%user_id, "Subscription gracefully shutdown");
                        Ok(())
                    }
                    _ => process_msgs(&mut session, &mut ctx, &mut sub, &payload).await
                },
                process_result = process_msgs(&mut session, &mut ctx, &mut sub, &payload) => {
                    tracing::debug!(%user_id, "Process messages completed");
                    process_result
                }
            };

            if let Err(err) = result {
                tracing::error!(%user_id, error = %err, "Subscription processing error");
                let _ = session.close(Some(CloseReason::from(err))).await;
            }
        }
    })
}

async fn process_msgs(
    session: &mut Session,
    ctx: &mut WsController,
    sub: &mut BoxedStream,
    payload: &SubscriptionPayload,
) -> Result<(), WebsocketError> {
    while let Some(result) = sub.next().await {
        let result = result?;
        let payload = decode_and_response(payload.to_owned(), result).await?;
        ctx.send_message(session, payload).await?;
    }

    ctx.remove_subscription(payload).await;
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
