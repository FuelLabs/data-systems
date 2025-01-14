use std::{pin::pin, sync::Arc, time::Instant};

use actix_web::{
    web::{self, Bytes},
    HttpRequest,
    Responder,
};
use actix_ws::{CloseCode, CloseReason, Message, MessageStream, Session};
use fuel_streams_core::FuelStreams;
use fuel_web_utils::telemetry::Telemetry;
use futures::{
    future::{self, Either},
    StreamExt as _,
};
use uuid::Uuid;

use crate::{
    metrics::Metrics,
    server::{
        errors::WebsocketError,
        state::ServerState,
        types::ClientMessage,
        websocket::{subscribe, unsubscribe, WsController},
    },
};

#[derive(Debug)]
enum CloseAction {
    Error(WebsocketError),
    Closed(Option<CloseReason>),
    Unsubscribe,
    Timeout,
}
impl From<CloseAction> for CloseReason {
    fn from(action: CloseAction) -> Self {
        match action {
            CloseAction::Closed(reason) => {
                reason.unwrap_or(CloseCode::Normal.into())
            }
            CloseAction::Error(_) => CloseCode::Away.into(),
            CloseAction::Unsubscribe => CloseCode::Normal.into(),
            CloseAction::Timeout => CloseCode::Away.into(),
        }
    }
}

pub async fn get_websocket(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<ServerState>,
) -> actix_web::Result<impl Responder> {
    let user_id = WsController::user_id_from_req(&req)?;
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    let fuel_streams = state.fuel_streams.clone();
    let telemetry = state.telemetry.clone();
    actix_web::rt::spawn(handler(
        session,
        msg_stream,
        telemetry,
        fuel_streams,
        user_id,
    ));
    Ok(response)
}

async fn handler(
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    telemetry: Arc<Telemetry<Metrics>>,
    fuel_streams: Arc<FuelStreams>,
    user_id: Uuid,
) -> Result<(), WebsocketError> {
    let mut ctx = WsController::new(user_id, telemetry, fuel_streams);
    tracing::info!(
        %user_id,
        event = "websocket_connection_opened",
        "WebSocket connection opened"
    );

    let action = handle_messages(&mut ctx, &mut session, msg_stream).await;
    if let Some(action) = action {
        if let CloseAction::Error(err) = &action {
            ctx.send_error_msg(&mut session, err).await?;
        }
        ctx.close_session(session, action.into()).await;
    }
    Ok(())
}

async fn handle_messages(
    ctx: &mut WsController,
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
                    match handle_client_msg(session, ctx, msg).await {
                        Err(err) => break Some(CloseAction::Error(err)),
                        Ok(Some(close_action)) => break Some(close_action),
                        Ok(None) => {}
                    }
                }
                Message::Binary(msg) => {
                    match handle_client_msg(session, ctx, msg).await {
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
                    break Some(CloseAction::Closed(reason));
                }
                Message::Continuation(_) => {
                    let user_id = ctx.user_id();
                    tracing::warn!(%user_id, "Continuation frames not supported");
                    let err = WebsocketError::UnsupportedMessageType;
                    break Some(CloseAction::Error(err));
                }
                Message::Nop => {}
            },
            Either::Left((Some(Err(err)), _)) => {
                let user_id = ctx.user_id();
                tracing::error!(%user_id, error = %err, "WebSocket protocol error");
                break Some(CloseAction::Error(WebsocketError::from(err)));
            }
            Either::Left((None, _)) => {
                let user_id = ctx.user_id();
                tracing::info!(%user_id, "Client disconnected");
                break None;
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

async fn handle_client_msg(
    session: &mut Session,
    ctx: &mut WsController,
    msg: Bytes,
) -> Result<Option<CloseAction>, WebsocketError> {
    tracing::info!("Received binary {:?}", msg);
    let msg = serde_json::from_slice(&msg)?;
    match msg {
        ClientMessage::Subscribe(payload) => {
            subscribe(session, ctx, payload).await?;
            Ok(None)
        }
        ClientMessage::Unsubscribe(payload) => {
            unsubscribe(session, ctx, payload).await?;
            Ok(Some(CloseAction::Unsubscribe))
        }
    }
}
