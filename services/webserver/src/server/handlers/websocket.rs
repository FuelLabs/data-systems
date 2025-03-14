use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use fuel_streams_core::server::ServerRequest;
use fuel_web_utils::api_key::ApiKey;
use futures::StreamExt;
use tokio::sync::mpsc::Receiver;

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
    Closed(axum::extract::ws::CloseFrame),
    Disconnect,
    Timeout,
}

impl From<&CloseAction> for Option<axum::extract::ws::CloseFrame> {
    fn from(action: &CloseAction) -> Self {
        match action {
            CloseAction::Closed(frame) => Some(frame.clone()),
            CloseAction::Disconnect => Some(axum::extract::ws::CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: String::new().into(),
            }),
            CloseAction::Error(_) => Some(axum::extract::ws::CloseFrame {
                code: axum::extract::ws::close_code::AWAY,
                reason: String::new().into(),
            }),
            CloseAction::Timeout => Some(axum::extract::ws::CloseFrame {
                code: axum::extract::ws::close_code::AWAY,
                reason: String::new().into(),
            }),
        }
    }
}

pub async fn get_websocket(
    State(state): State<ServerState>,
    ws: WebSocketUpgrade,
    req: axum::http::Request<axum::body::Body>,
) -> impl IntoResponse {
    match ApiKey::from_req(&req) {
        Ok(api_key) => ws.on_upgrade(move |socket| async move {
            if let Err(e) = handler(socket, api_key, state).await {
                tracing::error!("WebSocket handler error: {:?}", e);
            }
        }),
        Err(e) => {
            tracing::error!("API key error: {:?}", e);
            axum::http::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::empty())
                .unwrap()
        }
    }
}

async fn handler(
    socket: WebSocket,
    api_key: ApiKey,
    state: ServerState,
) -> Result<(), WebsocketError> {
    let streams = state.fuel_streams.to_owned();
    let telemetry = state.telemetry.to_owned();
    let rate_limiter = state.api_keys_manager.rate_limiter().to_owned();
    let connection_checker = state.connection_checker.to_owned();
    let (tx, rx) = tokio::sync::mpsc::channel::<ConnectionSignal>(2);
    connection_checker.register(&api_key, tx).await;

    let ctx =
        WsSession::new(&api_key, telemetry, streams, rate_limiter, socket);

    tracing::info!(
        %api_key,
        event = "websocket_connection_opened",
        "WebSocket connection opened"
    );

    let action = handle_messages(&ctx, &connection_checker, rx).await?;
    if let Some(action) = action {
        ctx.close_session(&action).await
    } else {
        Ok(())
    }
}

async fn handle_messages(
    ctx: &WsSession,
    connection_checker: &Arc<ConnectionChecker>,
    mut signal_rx: Receiver<ConnectionSignal>,
) -> Result<Option<CloseAction>, WebsocketError> {
    let api_key_id = ctx.api_key();
    let mut receiver = ctx.socket_receiver.lock().await;
    loop {
        tokio::select! {
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        let encoded = text.as_bytes().to_vec().into();
                        match handle_websocket_request(ctx, encoded).await {
                            Err(err) => {
                                let action = CloseAction::Error(err);
                                return Ok(Some(action));
                            }
                            Ok(Some(close_action)) => {
                                return Ok(Some(close_action));
                            }
                            Ok(None) => {
                                connection_checker.update_heartbeat(api_key_id).await;
                                continue;
                            }
                        }
                    }
                    Ok(Message::Binary(bin)) => {
                        match handle_websocket_request(ctx, bin).await {
                            Err(err) => {
                                let action = CloseAction::Error(err);
                                return Ok(Some(action));
                            }
                            Ok(Some(close_action)) => {
                                return Ok(Some(close_action));
                            }
                            Ok(None) => {
                                connection_checker.update_heartbeat(api_key_id).await;
                                continue;
                            }
                        }
                    }
                    Ok(Message::Close(reason)) => {
                        tracing::info!(api_key = %ctx.api_key(), close_frame = ?reason, "Client sent close frame");
                        let action = match reason {
                            Some(frame) => CloseAction::Closed(frame),
                            None => CloseAction::Disconnect,
                        };
                        return Ok(Some(action));
                    }
                    Ok(Message::Ping(data)) => {
                        tracing::debug!(api_key = %ctx.api_key(), "Received client ping: {:?}", data);
                        connection_checker.update_heartbeat(api_key_id).await;
                    }
                    Ok(Message::Pong(data)) => {
                        tracing::debug!(api_key = %ctx.api_key(), "Received client pong: {:?}", data);
                        connection_checker.update_heartbeat(api_key_id).await;
                    }
                    Err(err) => {
                        tracing::error!(api_key = %ctx.api_key(), error = %err, "WebSocket protocol error");
                        let action = CloseAction::Error(WebsocketError::from(err));
                        return Ok(Some(action));
                    }
                }
            }
            Some(signal) = signal_rx.recv() => {
                match signal {
                    ConnectionSignal::Ping => {
                        let message = Message::Ping(axum::body::Bytes::new());
                        if let Err(err) = ctx.send_socket_message(message).await {
                            match err {
                                WebsocketError::ClosedWithReason { .. } => {
                                    return Ok(None);
                                }
                                WebsocketError::Closed(_) => {
                                    return Ok(None);
                                }
                                _ => {
                                    tracing::error!(api_key = %ctx.api_key(), "Failed to send ping: {}", err);
                                    let action = CloseAction::Timeout;
                                    return Ok(Some(action));
                                }
                            }
                        }
                    }
                    ConnectionSignal::Timeout => {
                        tracing::info!(api_key = %ctx.api_key(), "Heartbeat timeout detected");
                        let action = CloseAction::Timeout;
                        return Ok(Some(action));
                    }
                }
            }
            else => {
                tracing::info!(api_key = %ctx.api_key(), "All streams closed, client disconnected");
                let action = CloseAction::Disconnect;
                return Ok(Some(action));
            }
        }
    }
}

async fn handle_websocket_request(
    ctx: &WsSession,
    msg: axum::body::Bytes,
) -> Result<Option<CloseAction>, WebsocketError> {
    tracing::info!("Received message: {:?}", msg);
    let server_request: ServerRequest = msg.as_ref().try_into()?;
    match server_request {
        ServerRequest::Subscribe(_) => {
            subscribe_mult(ctx, &server_request).await?;
            Ok(None)
        }
        ServerRequest::Unsubscribe(_) => {
            unsubscribe_mult(ctx, &server_request).await?;
            Ok(None)
        }
    }
}
