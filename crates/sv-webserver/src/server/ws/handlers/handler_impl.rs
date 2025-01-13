use std::sync::atomic::AtomicUsize;

use actix_web::{
    web::{self, Bytes},
    HttpMessage,
    HttpRequest,
    Responder,
};
use actix_ws::Message;
use uuid::Uuid;

use crate::server::{
    state::ServerState,
    ws::handlers::{
        close::handle_close,
        message::handle_message,
        ping_pong::{handle_ping, handle_pong},
        unknown::handle_unknown,
    },
};

static _NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub async fn get_ws(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<ServerState>,
) -> actix_web::Result<impl Responder> {
    // extract user id
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

    // split the request into response, session, and message stream
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // spawm an actor handling the ws connection
    let streams = state.fuel_streams.clone();
    let telemetry = state.telemetry.clone();

    actix_web::rt::spawn(async move {
        tracing::info!("Ws opened for user id {:?}", user_id.to_string());
        while let Some(Ok(msg)) = msg_stream.recv().await {
            let mut session = session.clone();
            match msg {
                Message::Ping(bytes) => handle_ping(&mut session, &bytes).await,
                Message::Pong(bytes) => handle_pong(&bytes),
                Message::Text(string) => {
                    let bytes = Bytes::from(string.as_bytes().to_vec());
                    let _ = handle_message(
                        bytes, user_id, session, &telemetry, &streams,
                    )
                    .await;
                }
                Message::Binary(bytes) => {
                    let _ = handle_message(
                        bytes, user_id, session, &telemetry, &streams,
                    )
                    .await;
                }
                Message::Close(reason) => {
                    handle_close(reason, user_id, session, &telemetry).await;
                }
                _ => handle_unknown(user_id, session, &telemetry).await,
            };
        }
    });

    Ok(response)
}
