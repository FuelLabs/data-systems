use axum::extract::ws::Message;
use fuel_streams_core::{
    server::ServerResponse,
    types::{ServerRequest, StreamResponse},
    BoxedStream,
    StreamError,
};
use fuel_web_utils::api_key::ApiKey;
use futures::{
    stream::{SelectAll, StreamExt},
    SinkExt,
};
use smallvec::SmallVec;

use crate::server::{
    errors::WebsocketError,
    handlers::websocket::CloseAction,
    websocket::WsSession,
};

type CombinedStream = SelectAll<
    Box<
        dyn futures::Stream<Item = Result<StreamResponse, StreamError>>
            + Send
            + Unpin,
    >,
>;

pub async fn subscribe_mult(
    ctx: &WsSession,
    server_request: &ServerRequest,
) -> Result<(), WebsocketError> {
    let api_key = ctx.api_key().clone();
    let subscriptions = server_request.subscriptions(&api_key);
    let mut streams = SmallVec::<[BoxedStream; 20]>::new();
    let mut subscribed_msgs: SmallVec<[ServerResponse; 20]> = SmallVec::new();

    for subscription in subscriptions {
        tracing::info!(
            "Received subscribe message: {:?}",
            &subscription.payload
        );
        let api_key_role = api_key.role();
        let sub = ctx
            .streams
            .subscribe_by_entity(api_key_role, &subscription)
            .await?;
        subscribed_msgs
            .push(ServerResponse::Subscribed(subscription.to_owned()));
        ctx.add_subscription(&subscription).await?;
        streams.push(sub);
    }

    if !subscribed_msgs.is_empty() {
        let combined_stream = futures::stream::select_all(streams);
        send_subscribed_msgs(ctx, &subscribed_msgs).await?;
        tokio::spawn({
            let api_key = api_key.clone();
            let ctx = ctx.clone();
            async move {
                process_subscription(&ctx, &api_key, combined_stream).await
            }
        });
    }

    tracing::info!(%api_key, "Subscription processing completed");
    Ok(())
}

async fn send_subscribed_msgs(
    ctx: &WsSession,
    subscribed_msgs: &[ServerResponse],
) -> Result<(), WebsocketError> {
    let msg_encoded =
        serde_json::to_vec(&subscribed_msgs).map_err(WebsocketError::Serde)?;
    let msg_encoded = axum::body::Bytes::from(msg_encoded);
    {
        let mut sender = ctx.socket_sender.lock().await;
        sender.send(Message::Binary(msg_encoded)).await?;
    }
    Ok(())
}

async fn process_subscription(
    ctx: &WsSession,
    api_key: &ApiKey,
    mut stream: CombinedStream,
) -> Result<(), WebsocketError> {
    let mut shutdown_rx = ctx.receiver();
    tracing::debug!(%api_key, "Starting subscription task, initial shutdown_rx: {}", *shutdown_rx.borrow());
    loop {
        tokio::select! {
            result = stream.next() => {
                match result {
                    Some(Ok(result)) => {
                        tracing::debug!("Received message from stream: {:?}", result);
                        let payload = ServerResponse::Response(result);
                        if let Err(err) = ctx.send_message(payload).await {
                            match err {
                                WebsocketError::ClosedWithReason { .. } => {
                                    return Ok(());
                                }
                                WebsocketError::Closed(_) => {
                                    return Ok(());
                                }
                                _ => {
                                    tracing::error!(api_key = %ctx.api_key(), "Failed to send message: {}", err);
                                    ctx.close_session(&CloseAction::Error(err)).await?;
                                    return Ok(());
                                }
                            }
                        }
                    }
                    Some(Err(err)) => {
                        tracing::error!(%api_key, "Stream error: {}", err);
                        ctx.close_session(&CloseAction::Error(err.into())).await?;
                        return Ok(());
                    }
                    None => {
                        tracing::info!(%api_key, "All streams ended, cleaning up subscriptions");
                        return Ok(());
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                if !*shutdown_rx.borrow() {
                    tracing::info!(%api_key, "Received shutdown signal, exiting subscription task");
                    return Ok(());
                }
            }
        }
    }
}
