use std::sync::Arc;

use actix_web::{
    web::{self, Bytes},
    HttpRequest,
    Responder,
};
use actix_ws::{CloseCode, CloseReason, Message, MessageStream, Session};
use fuel_streams_core::server::ServerRequest;
use fuel_web_utils::api_key::ApiKey;
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::server::{
    errors::WebsocketError,
    state::ServerState,
    websocket::{
        subscribe_mult,
        unsubscribe_mult,
        ConnectionChecker,
        ConnectionSignal,
        WsSession,
    },
};

#[derive(Debug)]
pub enum CloseAction {
    Error(WebsocketError),
    Closed(CloseReason),
    Disconnect,
    Timeout,
}

impl From<&CloseAction> for CloseReason {
    fn from(action: &CloseAction) -> Self {
        match action {
            CloseAction::Closed(reason) => reason.clone(),
            CloseAction::Disconnect => CloseCode::Normal.into(),
            CloseAction::Error(_) => CloseCode::Away.into(),
            CloseAction::Timeout => CloseCode::Away.into(),
        }
    }
}

pub async fn get_websocket(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<ServerState>,
) -> actix_web::Result<impl Responder> {
    let api_key = ApiKey::from_req(&req)?;
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    actix_web::rt::spawn(handler(session, msg_stream, api_key, state));
    Ok(response)
}

async fn handler(
    mut session: Session,
    msg_stream: MessageStream,
    api_key: ApiKey,
    state: web::Data<ServerState>,
) -> Result<(), WebsocketError> {
    let streams = state.fuel_streams.to_owned();
    let telemetry = state.telemetry.to_owned();
    let rate_limiter = state.api_keys_manager.rate_limiter().to_owned();
    let connection_checker = state.connection_checker.to_owned();
    let mut ctx = WsSession::new(&api_key, telemetry, streams, rate_limiter);
    let (tx, signal_rx) = mpsc::channel::<ConnectionSignal>(2);
    connection_checker.register(ctx.to_owned(), tx).await;
    tracing::info!(
        %api_key,
        event = "websocket_connection_opened",
        "WebSocket connection opened"
    );

    let action = handle_messages(
        &mut ctx,
        &mut session,
        msg_stream,
        &connection_checker,
        signal_rx,
    )
    .await;

    if let Some(ref action) = action {
        match action {
            CloseAction::Error(err) => {
                ctx.send_error_msg(&mut session, err).await?;
                ctx.clone().close_session(session, action).await;
            }
            _ => {
                ctx.clone().close_session(session, action).await;
            }
        }
    }

    connection_checker
        .unregister(&api_key.id().to_string())
        .await;

    Ok(())
}

async fn handle_messages(
    ctx: &mut WsSession,
    session: &mut Session,
    msg_stream: MessageStream,
    connection_checker: &Arc<ConnectionChecker>,
    mut signal_rx: mpsc::Receiver<ConnectionSignal>,
) -> Option<CloseAction> {
    let api_key_id = ctx.api_key().id().to_string();
    let mut msg_stream = msg_stream.max_frame_size(ctx.max_frame_size());

    let mut shutdown_rx = ctx.receiver();
    loop {
        tokio::select! {
            Some(msg_result) = msg_stream.next() => {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if text.trim().eq_ignore_ascii_case("disconnect") {
                            let api_key = ctx.api_key();
                            tracing::info!(%api_key, "Client requested disconnect");
                            ctx.shutdown().await;
                            return Some(CloseAction::Disconnect);
                        }
                        let msg = Bytes::from(text.as_bytes().to_vec());
                        match handle_websocket_request(session, ctx, msg).await {
                            Err(err) => return Some(CloseAction::Error(err)),
                            Ok(Some(close_action)) => return Some(close_action),
                            Ok(None) => {
                                connection_checker
                                    .update_heartbeat(&api_key_id)
                                    .await;
                            }
                        }
                    }
                    Ok(Message::Binary(bin)) => {
                        match handle_websocket_request(session, ctx, bin).await {
                            Err(err) => return Some(CloseAction::Error(err)),
                            Ok(Some(close_action)) => return Some(close_action),
                            Ok(None) => {
                                connection_checker
                                    .update_heartbeat(&api_key_id)
                                    .await;
                            }
                        }
                    }
                    Ok(Message::Close(reason)) => {
                        let api_key = ctx.api_key();
                        tracing::info!(%api_key, "Client sent close frame");
                        let close_action = match reason {
                            Some(reason) => CloseAction::Closed(reason),
                            None => CloseAction::Disconnect,
                        };
                        ctx.shutdown().await;
                        return Some(close_action);
                    }
                    Ok(Message::Ping(data)) => {
                        tracing::debug!(api_key = %ctx.api_key(), "Received client ping: {:?}", data);
                        connection_checker.update_heartbeat(&api_key_id).await;
                    }
                    Ok(Message::Pong(data)) => {
                        tracing::debug!(api_key = %ctx.api_key(), "Received client pong: {:?}", data);
                        connection_checker.update_heartbeat(&api_key_id).await;
                    }
                    Ok(Message::Continuation(_)) => {
                        tracing::debug!(api_key = %ctx.api_key(), "Received client continuation");
                        connection_checker.update_heartbeat(&api_key_id).await;
                    }
                    Ok(Message::Nop) => {
                        tracing::debug!(api_key = %ctx.api_key(), "Received client nop");
                        connection_checker.update_heartbeat(&api_key_id).await;
                    }
                    Err(err) => {
                        let api_key = ctx.api_key();
                        tracing::error!(%api_key, error = %err, "WebSocket protocol error");
                        ctx.shutdown().await;
                        return Some(CloseAction::Error(WebsocketError::from(err)));
                    }
                }
            }
            Some(signal) = signal_rx.recv() => {
                let api_key = ctx.api_key();
                match signal {
                    ConnectionSignal::Ping => {
                        if let Err(err) = session.ping(b"").await {
                            tracing::error!(%api_key, "Failed to send ping: {}", err);
                            ctx.shutdown().await;
                            return Some(CloseAction::Error(
                                WebsocketError::SendError,
                            ));
                        }
                    }
                    ConnectionSignal::Timeout => {
                        tracing::info!(%api_key, "Heartbeat timeout detected");
                        ctx.shutdown().await;
                        return Some(CloseAction::Timeout);
                    }
                }
            }
            // Watch for shutdown signal
            _ = shutdown_rx.changed() => {
                if !*shutdown_rx.borrow() {
                    // Shutdown signal (false)
                    let api_key = ctx.api_key();
                    tracing::info!(%api_key, "Subscription task requested closure");
                    return Some(CloseAction::Error(WebsocketError::SendError));
                }
            }
            else => {
                // All streams have ended
                let api_key = ctx.api_key();
                tracing::info!(%api_key, "All streams closed, client disconnected");
                ctx.shutdown().await;
                return Some(CloseAction::Disconnect);
            }
        }
    }
}

async fn handle_websocket_request(
    session: &mut Session,
    ctx: &mut WsSession,
    msg: Bytes,
) -> Result<Option<CloseAction>, WebsocketError> {
    tracing::info!("Received binary {:?}", msg);
    let server_request: ServerRequest = msg.as_ref().try_into()?;
    match server_request {
        ServerRequest::Subscribe(_) => {
            subscribe_mult(session, ctx, &server_request).await?;
            Ok(None)
        }
        ServerRequest::Unsubscribe(_) => {
            unsubscribe_mult(session, ctx, &server_request).await?;
            Ok(None)
        }
    }
}
