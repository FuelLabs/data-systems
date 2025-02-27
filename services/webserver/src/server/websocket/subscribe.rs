use std::sync::Arc;

use actix_ws::{CloseReason, Session};
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
use futures::{future::try_join_all, StreamExt};

use crate::server::{
    errors::WebsocketError,
    websocket::{unsubscribe, WsSession},
};

pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let subscriptions = server_request.subscriptions(ctx.api_key());
    let handles: Vec<_> = subscriptions
        .into_iter()
        .map(|subscription| {
            let mut ctx = ctx.clone();
            let mut session = session.clone();
            actix_web::rt::spawn(async move {
                subscribe(&mut session, &mut ctx, &subscription).await
            })
        })
        .collect();

    let api_key = ctx.api_key();
    match try_join_all(handles).await {
        Ok(results) => {
            if let Some(err) = results.into_iter().find_map(|r| r.err()) {
                tracing::error!(%api_key, "Subscription task failed: {}", err);
                return Err(WebsocketError::Subscribe(format!(
                    "Subscription task failed: {err}"
                )));
            }
            tracing::info!(%api_key, "Subscriptions running");
            Ok(())
        }
        Err(err) => {
            tracing::error!(%api_key, "Subscriptions failed: {}", err);
            Err(WebsocketError::Subscribe(format!(
                "Subscriptions failed: {err}"
            )))
        }
    }
}

async fn subscribe(
    session: &mut Session,
    ctx: &mut WsSession,
    subscription: &Subscription,
) -> Result<(), WebsocketError> {
    let payload = &subscription.payload;
    tracing::info!("Received subscribe message: {:?}", payload);
    if ctx
        .check_duplicate_subscription(session, subscription)
        .await?
    {
        return Ok(());
    }

    // Subscribe to the subject
    let api_key_role = ctx.api_key().role();
    let sub =
        create_subscriber(api_key_role, &ctx.streams, subscription).await?;
    let subscribed_msg = ServerResponse::Subscribed(subscription.clone());
    ctx.send_message(session, subscribed_msg).await?;
    ctx.add_subscription(subscription).await?;

    // Spawn a task to process messages
    actix_web::rt::spawn({
        let mut session = session.to_owned();
        let mut ctx = ctx.to_owned();
        let subscription = subscription.clone();
        let api_key = ctx.api_key().clone();
        async move {
            let result = process_subscription(
                &mut session,
                &mut ctx,
                &subscription,
                sub,
            )
            .await;
            if let Err(err) = result {
                tracing::error!(%api_key, error = %err, "Subscription processing error");
                let _ = session.close(Some(CloseReason::from(err))).await;
            }
        }
    });

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
