use std::sync::Arc;

use actix_ws::{CloseReason, Session};
use fuel_streams_core::{
    prelude::{IntoSubject, SubjectPayload},
    server::{DeliverPolicy, ServerResponse, Subscription},
    types::{MessagePayload, ServerRequest, StreamMessage},
    BoxedStream,
    FuelStreams,
};
use fuel_streams_domains::Subjects;
use fuel_streams_store::record::RecordEntity;
use fuel_web_utils::server::api::API_VERSION;
use futures::{future::try_join_all, StreamExt};

use crate::server::{errors::WebsocketError, websocket::WsSession};

pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let subjects = server_request.subscribe.clone();
    let deliver_policy = server_request.deliver_policy.clone();
    if subjects.is_empty() {
        tracing::debug!("No subscriptions requested");
        return Ok(());
    }

    let handles: Vec<_> = subjects
        .into_iter()
        .map(|payload| {
            let mut ctx = ctx.clone();
            let mut session = session.clone();
            let api_key = ctx.api_key().clone();
            let subscription =
                Subscription::new(&api_key, &deliver_policy, &payload);
            actix_web::rt::spawn(async move {
                subscribe(&mut session, &mut ctx, &subscription.into()).await
            })
        })
        .collect();

    match try_join_all(handles).await {
        Ok(_) => {
            tracing::info!("All subscriptions completed successfully");
            Ok(())
        }
        Err(err) => {
            tracing::error!("Subscription task failed: {}", err);
            Err(WebsocketError::Subscribe(format!(
                "Subscription task failed: {err}"
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
    let sub = create_subscriber(
        &ctx.streams,
        &subscription.payload,
        subscription.deliver_policy,
    )
    .await?;

    // Send the subscription message to the client
    let subscribed_msg =
        ServerResponse::Subscribed(subscription.payload.clone());
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
    let payload = &subscription.payload;
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
    let payload = &subscription.payload;
    tracing::debug!(?payload, "Starting to process messages");
    while let Some(result) = sub.next().await {
        let result = result?;
        tracing::debug!(?payload, ?result, "Received message from stream");
        let payload = decode_and_respond(payload.clone(), result).await?;
        tracing::debug!("Sending message to client: {:?}", payload);
        ctx.send_message(session, payload).await?;
    }

    tracing::debug!(?payload, "Stream ended, removing subscription");
    ctx.remove_subscription(subscription).await;
    Ok(())
}

async fn create_subscriber(
    streams: &Arc<FuelStreams>,
    subject_payload: &SubjectPayload,
    deliver_policy: DeliverPolicy,
) -> Result<BoxedStream, WebsocketError> {
    let subject: Subjects = subject_payload.clone().try_into()?;
    let subject: Arc<dyn IntoSubject> = subject.into();
    let subject_id = subject_payload.subject.as_str();
    let record_entity = RecordEntity::try_from(subject_id)?;
    let stream = match record_entity {
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

pub async fn decode_and_respond(
    subject_payload: SubjectPayload,
    (subject, data): (String, Vec<u8>),
) -> Result<ServerResponse, WebsocketError> {
    let subject_id = subject_payload.subject.as_str();
    let data = MessagePayload::new(subject_id, &data)?;
    let response_message = StreamMessage {
        subject,
        ty: subject_id.to_string(),
        version: API_VERSION.to_string(),
        payload: data,
    };
    Ok(ServerResponse::Response(response_message))
}
