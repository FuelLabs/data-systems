use std::{pin::pin, sync::Arc, time::Instant};

use actix_web::{
    web::{self, Bytes},
    HttpRequest,
    Responder,
};
use actix_ws::{CloseCode, CloseReason, Message, MessageStream, Session};
use fuel_streams_core::{server::ServerRequest, FuelStreams};
use fuel_web_utils::{
    api_key::{rate_limiter::RateLimitsController, ApiKey},
    telemetry::Telemetry,
};
use futures::{
    future::{self, Either},
    StreamExt as _,
};

use crate::{
    metrics::Metrics,
    server::{
        errors::WebsocketError,
        state::ServerState,
        websocket::{subscribe_mult, unsubscribe_mult, WsSession},
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
    let fuel_streams = state.fuel_streams.clone();
    let telemetry = state.telemetry.clone();
    let rate_limiter = state.api_keys_manager.rate_limiter().clone();
    actix_web::rt::spawn(handler(
        session,
        msg_stream,
        telemetry,
        fuel_streams,
        api_key,
        rate_limiter,
    ));
    Ok(response)
}

async fn handler(
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    telemetry: Arc<Telemetry<Metrics>>,
    fuel_streams: Arc<FuelStreams>,
    api_key: ApiKey,
    rate_limiter: Arc<RateLimitsController>,
) -> Result<(), WebsocketError> {
    let mut ctx =
        WsSession::new(&api_key, telemetry, fuel_streams, rate_limiter);
    tracing::info!(
        %api_key,
        event = "websocket_connection_opened",
        "WebSocket connection opened"
    );

    let action = handle_messages(&mut ctx, &mut session, msg_stream).await;
    if let Some(action) = &action {
        match action {
            CloseAction::Error(err) => {
                ctx.send_error_msg(&mut session, err).await?;
                ctx.close_session(session, action).await;
            }
            _ => {
                ctx.close_session(session, action).await;
            }
        }
    };
    Ok(())
}

async fn handle_messages(
    ctx: &mut WsSession,
    session: &mut Session,
    msg_stream: MessageStream,
) -> Option<CloseAction> {
    let mut last_heartbeat = Instant::now();
    let mut interval = tokio::time::interval(ctx.heartbeat_interval());
    let mut msg_stream = msg_stream.max_frame_size(ctx.max_frame_size());
    let mut msg_stream = pin!(msg_stream);

    loop {
        let tick = pin!(interval.tick());
        match future::select(msg_stream.next(), tick).await {
            Either::Left((Some(Ok(msg)), _)) => match msg {
                Message::Text(msg) => {
                    let msg = Bytes::from(msg.as_bytes().to_vec());
                    match handle_websocket_request(session, ctx, msg).await {
                        Err(err) => break Some(CloseAction::Error(err)),
                        Ok(Some(close_action)) => break Some(close_action),
                        Ok(None) => {}
                    }
                }
                Message::Binary(msg) => {
                    match handle_websocket_request(session, ctx, msg).await {
                        Err(err) => break Some(CloseAction::Error(err)),
                        Ok(Some(close_action)) => break Some(close_action),
                        Ok(None) => {}
                    }
                }
                Message::Ping(bytes) => {
                    last_heartbeat = Instant::now();
                    if let Err(err) = session.pong(&bytes).await {
                        let err = WebsocketError::from(err);
                        break Some(CloseAction::Error(err));
                    }
                }
                Message::Pong(_) => {
                    last_heartbeat = Instant::now();
                }
                Message::Close(reason) => {
                    break match reason {
                        Some(reason) => Some(CloseAction::Closed(reason)),
                        None => Some(CloseAction::Disconnect),
                    }
                }
                Message::Continuation(_) => {
                    let api_key = ctx.api_key();
                    tracing::warn!(%api_key, "Continuation frames not supported");
                    let err = WebsocketError::UnsupportedMessageType;
                    break Some(CloseAction::Error(err));
                }
                Message::Nop => {}
            },
            Either::Left((Some(Err(err)), _)) => {
                let api_key = ctx.api_key();
                tracing::error!(%api_key, error = %err, "WebSocket protocol error");
                break Some(CloseAction::Error(WebsocketError::from(err)));
            }
            Either::Left((None, _)) => {
                let api_key = ctx.api_key();
                tracing::info!(%api_key, "Client disconnected");
                break Some(CloseAction::Disconnect);
            }
            Either::Right((_inst, _)) => {
                if let Err(err) = ctx.heartbeat(session, last_heartbeat).await {
                    match err {
                        WebsocketError::Timeout => {
                            break Some(CloseAction::Timeout)
                        }
                        _ => break Some(CloseAction::Error(err)),
                    }
                }
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
