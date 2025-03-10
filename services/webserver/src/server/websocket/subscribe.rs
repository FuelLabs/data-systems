use std::sync::Arc;

use actix_ws::{CloseCode, CloseReason, Session};
use fuel_streams_core::{
    prelude::IntoSubject,
    server::{ServerResponse, Subscription},
    types::ServerRequest,
    BoxedStream,
    FuelStreams,
};
use fuel_streams_domains::Subjects;
use fuel_streams_store::record::RecordEntity;
use fuel_web_utils::api_key::ApiKeyRole;
use futures::StreamExt;
use tokio::sync::Semaphore;

use crate::server::{
    errors::WebsocketError,
    websocket::{unsubscribe, WsSession},
};

pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let semaphore = Arc::new(Semaphore::new(20));
    let subscriptions = server_request.subscriptions(ctx.api_key());
    let api_key = ctx.api_key();
    let mut join_set = tokio::task::JoinSet::new();
    for subscription in subscriptions {
        let payload = &subscription.payload;
        tracing::info!("Received subscribe message: {:?}", payload);
        if ctx.check_duplicated_sub(session, &subscription).await? {
            continue;
        }

        let api_key_role = ctx.api_key().role();
        let sub = create_subscriber(api_key_role, &ctx.streams, &subscription)
            .await?;
        let subscribed_msg = ServerResponse::Subscribed(subscription.clone());
        ctx.send_message(session, subscribed_msg).await?;
        ctx.add_subscription(&subscription).await?;

        let mut session_clone = session.clone();
        let mut ctx_clone = ctx.clone();
        let subscription_clone = subscription.clone();
        join_set.spawn({
            let semaphore = semaphore.clone();
            async move {
                let _permit = semaphore.acquire().await;
                process_subscription(
                    &mut session_clone,
                    &mut ctx_clone,
                    &subscription_clone,
                    sub,
                )
                .await
            }
        });
    }

    let mut shutdown_rx = ctx.receiver();
    actix_web::rt::spawn({
        let session = session.clone();
        let api_key = api_key.clone();
        async move {
            loop {
                tokio::select! {
                    Ok(shutdown_api_key) = shutdown_rx.recv() => {
                        if shutdown_api_key == api_key {
                            tracing::info!(%api_key, "Subscription gracefully shutdown");
                            break;
                        }
                    }
                    Some(result) = join_set.join_next() => {
                        let session = session.clone();
                        match result {
                            Ok(task_result) => {
                                if let Err(err) = task_result {
                                    tracing::error!(%api_key, "Subscription processing error: {}", err);
                                    let _ = session
                                        .close(Some(CloseReason::from(err)))
                                        .await;
                                }
                            }
                            Err(err) => {
                                tracing::error!(%api_key, "Subscription task failed: {}", err);
                                let _ = session
                                    .close(Some(CloseReason::from(CloseCode::Normal)))
                                    .await;
                            }
                        }
                    }
                    else => break,
                }
            }
        }
    });

    tracing::info!(%api_key, "All subscription tasks completed");
    Ok(())
}

async fn process_subscription(
    session: &mut Session,
    ctx: &mut WsSession,
    subscription: &Subscription,
    mut sub: BoxedStream,
) -> Result<(), WebsocketError> {
    let payload = subscription.payload.clone();
    let mut shutdown_rx = ctx.receiver();
    let api_key = ctx.api_key().clone();
    tracing::debug!(%api_key, ?payload, "Starting subscription process");
    loop {
        tokio::select! {
            Some(result) = sub.next() => {
                let result = result?;
                tracing::debug!(?payload, ?result, "Received message from stream");
                let payload = ServerResponse::Response(result);
                tracing::debug!("Sending message to client: {:?}", payload);
                ctx.send_message(session, payload).await?;
            }
            Ok(shutdown_api_key) = shutdown_rx.recv() => {
                if shutdown_api_key == api_key {
                    tracing::info!(%api_key, "Subscription gracefully shutdown");
                    return Ok(());
                }
            }
            else => {
                tracing::debug!(?payload, "Stream ended, removing subscription");
                return unsubscribe(session, ctx, subscription).await;
            }
        }
    }
}

async fn create_subscriber(
    api_key_role: &ApiKeyRole,
    streams: &Arc<FuelStreams>,
    subscription: &Subscription,
) -> Result<BoxedStream, WebsocketError> {
    let subject_payload = subscription.payload.clone();
    let deliver_policy = subscription.deliver_policy;
    let subject: Subjects = subject_payload.clone().try_into()?;
    let subject: Arc<dyn IntoSubject> = subject.into();
    let subject_id = subject_payload.subject.as_str();
    let record_entity = RecordEntity::try_from(subject_id)?;
    let stream = match record_entity {
        RecordEntity::Block => {
            streams
                .blocks
                .subscribe_dynamic(subject, deliver_policy, api_key_role)
                .await
        }
        RecordEntity::Transaction => {
            streams
                .transactions
                .subscribe_dynamic(subject, deliver_policy, api_key_role)
                .await
        }
        RecordEntity::Input => {
            streams
                .inputs
                .subscribe_dynamic(subject, deliver_policy, api_key_role)
                .await
        }
        RecordEntity::Output => {
            streams
                .outputs
                .subscribe_dynamic(subject, deliver_policy, api_key_role)
                .await
        }
        RecordEntity::Receipt => {
            streams
                .receipts
                .subscribe_dynamic(subject, deliver_policy, api_key_role)
                .await
        }
        RecordEntity::Utxo => {
            streams
                .utxos
                .subscribe_dynamic(subject, deliver_policy, api_key_role)
                .await
        }
    };
    Ok(Box::new(stream))
}
