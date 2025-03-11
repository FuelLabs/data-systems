use std::sync::Arc;

use actix_ws::Session;
use fuel_streams_core::{
    prelude::IntoSubject,
    server::{ServerResponse, Subscription},
    types::{ServerRequest, StreamResponse},
    BoxedStream,
    FuelStreams,
    StreamError,
};
use fuel_streams_domains::Subjects;
use fuel_streams_store::record::RecordEntity;
use fuel_web_utils::api_key::{ApiKey, ApiKeyRole};
use futures::stream::{SelectAll, StreamExt};
use smallvec::SmallVec;

use crate::server::{errors::WebsocketError, websocket::WsSession};

type CombinedStream = SelectAll<
    Box<
        dyn futures::Stream<Item = Result<StreamResponse, StreamError>>
            + Send
            + Unpin,
    >,
>;

pub async fn subscribe_mult(
    session: &mut Session,
    ctx: &mut WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let api_key = ctx.api_key();
    let subscriptions = server_request.subscriptions(api_key);
    let mut streams = SmallVec::<[BoxedStream; 20]>::new();
    let mut subscribed_msgs: SmallVec<[ServerResponse; 20]> = SmallVec::new();

    for subscription in subscriptions {
        tracing::info!(
            "Received subscribe message: {:?}",
            &subscription.payload
        );
        let api_key_role = api_key.role();
        let sub = create_subscriber(api_key_role, &ctx.streams, &subscription)
            .await?;
        subscribed_msgs
            .push(ServerResponse::Subscribed(subscription.to_owned()));
        ctx.add_subscription(&subscription).await?;
        streams.push(sub);
    }

    let combined_stream = futures::stream::select_all(streams);
    actix_web::rt::spawn({
        let ctx = ctx.to_owned();
        let api_key = api_key.to_owned();
        let mut session = session.to_owned();
        async move {
            if !subscribed_msgs.is_empty() {
                let msg_encoded = serde_json::to_vec(&subscribed_msgs)
                    .map_err(WebsocketError::Serde)?;
                session.binary(msg_encoded).await?;
                process_subscription(
                    &mut session,
                    &ctx,
                    &api_key,
                    combined_stream,
                )
                .await;
            }
            Ok::<(), WebsocketError>(())
        }
    });

    tracing::info!(%api_key, "Subscription task started for all subscriptions");
    Ok(())
}

async fn process_subscription(
    session: &mut Session,
    ctx: &WsSession,
    api_key: &ApiKey,
    mut stream: CombinedStream,
) {
    let mut shutdown_rx = ctx.receiver();
    loop {
        tokio::select! {
            Some(stream_result) = stream.next() => {
                match stream_result {
                    Ok(result) => {
                        tracing::debug!("Received message from stream: {:?}", result);
                        let payload = ServerResponse::Response(result);
                        if let Err(err) = ctx.send_message(session, payload).await {
                            match err {
                                WebsocketError::Closed(_) => {
                                    tracing::info!(%api_key, "Session closed, exiting subscription task");
                                    break;
                                }
                                err => {
                                    tracing::error!(%api_key, "Failed to send message: {}", err);
                                    ctx.shutdown().await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::error!(%api_key, "Stream error: {}", err);
                        ctx.shutdown().await;
                        break;
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                if !*shutdown_rx.borrow() {
                    tracing::info!(%api_key, "Received shutdown signal, exiting subscription task");
                    ctx.shutdown().await;
                    break;
                }
            }
            else => {
                tracing::info!(%api_key, "All streams ended, cleaning up subscriptions");
                ctx.shutdown().await;
                break;
            }
        }
    }

    tracing::info!(%api_key, "All streams ended or shutdown, cleaning up subscriptions");
    ctx.shutdown().await;
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
