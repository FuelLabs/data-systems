use std::sync::{atomic::AtomicUsize, Arc};

use actix_web::{
    web::{self, Bytes},
    HttpMessage,
    HttpRequest,
    Responder,
};
use actix_ws::{Message, MessageStream, Session};
use uuid::Uuid;

use super::{
    context::WsContext,
    errors::WsSubscriptionError,
    handler::handle_binary_message,
    models::ServerMessage,
};
use crate::server::state::ServerState;

static _NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub async fn get_ws(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<ServerState>,
) -> actix_web::Result<impl Responder> {
    // Extract user id
    let user_id = match req.extensions().get::<Uuid>() {
        Some(user_id) => {
            tracing::info!(
                "Authenticated WebSocket connection for user: {:?}",
                user_id.to_string()
            );
            user_id.to_owned()
        }
        None => {
            tracing::info!("Unauthenticated WebSocket connection");
            return Err(actix_web::error::ErrorUnauthorized(
                "Missing or invalid JWT",
            ));
        }
    };

    // Split the request into response, session, and message stream
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;

    // Record the new subscription
    state.context.telemetry.increment_subscriptions_count();

    // Spawn an actor handling the ws connection
    let streams = state.context.fuel_streams.clone();
    let telemetry = state.context.telemetry.clone();

    actix_web::rt::spawn(async move {
        handle_ws_connection(user_id, session, msg_stream, telemetry, streams)
            .await;
    });

    Ok(response)
}

async fn handle_ws_connection(
    user_id: Uuid,
    session: Session,
    mut msg_stream: MessageStream,
    telemetry: Arc<crate::telemetry::Telemetry>,
    streams: Arc<fuel_streams_core::FuelStreams>,
) {
    tracing::info!("WS opened for user id {:?}", user_id.to_string());

    while let Some(Ok(msg)) = msg_stream.recv().await {
        let mut session = session.clone();

        match msg {
            Message::Ping(bytes) => handle_ping(&mut session, &bytes).await,
            Message::Pong(bytes) => handle_pong(&bytes),
            Message::Text(string) => {
                handle_text(string, user_id, session, &telemetry, &streams)
                    .await
            }
            Message::Binary(bytes) => {
                handle_binary(bytes, user_id, session, &telemetry, &streams)
                    .await
            }
            Message::Close(reason) => {
                handle_close(reason, user_id, session, telemetry.clone()).await;
                return;
            }
            _ => {
                handle_unknown(user_id, session, telemetry.clone()).await;
                return;
            }
        }
    }
}

async fn handle_ping(session: &mut Session, bytes: &[u8]) {
    tracing::info!("Received ping, {:?}", bytes);
    if session.pong(bytes).await.is_err() {
        tracing::error!("Error sending pong, {:?}", bytes);
    }
}

fn handle_pong(bytes: &[u8]) {
    tracing::info!("Received pong, {:?}", bytes);
}

async fn handle_text(
    string: impl AsRef<str>,
    user_id: Uuid,
    session: Session,
    telemetry: &Arc<crate::telemetry::Telemetry>,
    streams: &Arc<fuel_streams_core::FuelStreams>,
) {
    let bytes = Bytes::from(string.as_ref().as_bytes().to_vec());
    handle_binary(bytes, user_id, session, telemetry, streams).await;
}

async fn handle_binary(
    bytes: Bytes,
    user_id: Uuid,
    session: Session,
    telemetry: &Arc<crate::telemetry::Telemetry>,
    streams: &Arc<fuel_streams_core::FuelStreams>,
) {
    let _ = handle_binary_message(bytes, user_id, session, telemetry, streams)
        .await;
}

async fn handle_close(
    reason: Option<actix_ws::CloseReason>,
    user_id: Uuid,
    session: Session,
    telemetry: Arc<crate::telemetry::Telemetry>,
) {
    tracing::info!(
        "Got close event, terminating session with reason {:?}",
        reason
    );
    let reason_str = reason.and_then(|r| r.description).unwrap_or_default();
    let ctx = WsContext::new(user_id, session, telemetry);
    ctx.close_with_error(WsSubscriptionError::ClosedWithReason(
        reason_str.to_string(),
    ))
    .await;
}

async fn handle_unknown(
    user_id: Uuid,
    session: Session,
    telemetry: Arc<crate::telemetry::Telemetry>,
) {
    tracing::error!("Received unknown message type");
    let ctx = WsContext::new(user_id, session, telemetry);
    ctx.close_with_error(WsSubscriptionError::ClosedWithReason(
        "Unknown message type".to_string(),
    ))
    .await;
}

/// Sends a message to the WebSocket
pub async fn send_message_to_socket(
    session: &mut Session,
    message: ServerMessage,
) {
    let data = serde_json::to_vec(&message).ok().unwrap_or_default();
    let _ = session.binary(data).await;
}
