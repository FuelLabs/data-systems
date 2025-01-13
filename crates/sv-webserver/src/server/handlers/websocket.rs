use std::sync::atomic::AtomicUsize;

use actix_web::{
    web::{self, Bytes},
    HttpRequest,
    Responder,
};
use actix_ws::Message;

use crate::server::{
    errors::WebsocketError,
    state::ServerState,
    subscriber::{subscribe, unsubscribe},
    types::ClientMessage,
    ws_context::WsContext,
};

static _NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub async fn get_ws(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<ServerState>,
) -> actix_web::Result<impl Responder> {
    let user_id = WsContext::user_id_from_req(&req)?;
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let streams = state.fuel_streams.clone();
    let telemetry = state.telemetry.clone();
    let ctx = WsContext::new(user_id, session.clone(), telemetry, streams);

    actix_web::rt::spawn(async move {
        tracing::info!("Ws opened for user id {:?}", user_id.to_string());
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Ping(bytes) => handle_ping(ctx.clone(), &bytes).await,
                Message::Pong(bytes) => handle_pong(&bytes),
                Message::Text(string) => {
                    handle_text(ctx.clone(), string.into()).await;
                }
                Message::Binary(bytes) => {
                    let _ = handle_message(bytes, ctx.clone()).await;
                }
                Message::Close(reason) => {
                    handle_close(reason, ctx.clone()).await;
                }
                _ => handle_unknown(ctx.clone()).await,
            };
        }
    });

    Ok(response)
}

async fn handle_ping(ctx: WsContext, bytes: &[u8]) {
    let mut session = ctx.session.clone();
    tracing::info!("Received ping, {:?}", bytes);
    if let Err(e) = ctx.handle_error(session.pong(bytes).await, true).await {
        tracing::error!("Error sending pong: {:?}", e);
    }
}

fn handle_pong(bytes: &[u8]) {
    tracing::info!("Received pong, {:?}", bytes);
}

async fn handle_text(ctx: WsContext, text: String) {
    let bytes = Bytes::from(text.as_bytes().to_vec());
    let _ = handle_message(bytes, ctx.clone()).await;
}

async fn handle_message(
    msg: Bytes,
    ctx: WsContext,
) -> Result<(), WebsocketError> {
    tracing::info!("Received binary {:?}", msg);
    let msg = serde_json::from_slice(&msg);
    let client_message = ctx.handle_error(msg, true).await?;
    match client_message {
        ClientMessage::Subscribe(payload) => {
            subscribe(payload, ctx.clone()).await
        }
        ClientMessage::Unsubscribe(payload) => {
            unsubscribe(ctx.clone(), payload).await
        }
    }
}

async fn handle_close(reason: Option<actix_ws::CloseReason>, ctx: WsContext) {
    tracing::info!(
        "Got close event, terminating session with reason {:?}",
        reason
    );
    let reason_str = reason.and_then(|r| r.description).unwrap_or_default();
    let _ = ctx
        .close_with_error(
            WebsocketError::ClosedWithReason(reason_str.to_string()),
            true,
        )
        .await;
}

async fn handle_unknown(ctx: WsContext) {
    tracing::error!("Received unknown message type");
    let reason =
        WebsocketError::ClosedWithReason("Unknown message type".to_string());
    let _ = ctx.close_with_error(reason, true).await;
}
