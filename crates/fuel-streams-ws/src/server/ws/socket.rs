use std::sync::atomic::AtomicUsize;

use actix_web::{
    web::{self, Bytes},
    HttpMessage,
    HttpRequest,
    Responder,
};
use actix_ws::{Message, Session};
use fuel_streams::{types::Block, StreamData, Streamable};
use uuid::Uuid;

use super::{
    errors::WsSubscriptionError,
    models::ClientMessage,
    streams::Streams,
};
use crate::server::{
    state::ServerState,
    ws::models::{ServerMessage, SubscriptionType},
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
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // increase the maximum allowed frame size to 1MiB and aggregate continuation frames
    // let mut msg_stream = msg_stream
    //     .max_frame_size(1024 * 1024)
    //     .aggregate_continuations();

    // record the new subscription
    state.context.telemetry.record_subscriptions_count();

    // spawm an actor handling the ws connection
    actix_web::rt::spawn(async move {
        tracing::info!("Ws opened for user id {:?}", user_id.to_string());
        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Ping(bytes) => {
                    tracing::info!("Received ping, {:?}", bytes);
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Pong(bytes) => {
                    tracing::info!("Received pong, {:?}", bytes);
                }
                Message::Text(string) => {
                    tracing::info!("Received text, {string}");
                }
                Message::Binary(bytes) => {
                    tracing::info!("Received binary {:?}", bytes);
                    let client_message = match parse_client_message(bytes) {
                        Ok(msg) => msg,
                        Err(e) => {
                            close_socket_with_error(e, session).await;
                            return;
                        }
                    };

                    // handle the client message
                    match client_message {
                        ClientMessage::Subscribe(payload) => {
                            tracing::info!(
                                "Received subscribe message: {:?}",
                                payload
                            );
                            let SubscriptionType::Stream(subject_wildcard) =
                                payload.topic;

                            // verify the subject name
                            if let Err(e) =
                                verify_subject_name(&subject_wildcard)
                            {
                                close_socket_with_error(e, session).await;
                                return;
                            }

                            // update metrics
                            state
                                .context
                                .telemetry
                                .update_streamer_success_metrics(
                                    user_id,
                                    &subject_wildcard,
                                );

                            // start the streamer async
                            let mut stream_session = session.clone();
                            let mut rx =
                                Streams::run_streamable_consumer::<Block>(
                                    state.context.client.clone(),
                                )
                                .await
                                .unwrap();

                            // receive in a background thread
                            actix_web::rt::spawn(async move {
                                while let Some(res) = rx.recv().await {
                                    let serialized_payload =
                                        match stream_to_server_message(res) {
                                            Ok(res) => res,
                                            Err(e) => {
                                                tracing::error!("Error serializing received stream message: {:?}", e);
                                                continue;
                                            }
                                        };
                                    let _ = stream_session
                                        .binary(serialized_payload)
                                        .await;
                                }
                            });
                        }
                        ClientMessage::Unsubscribe(payload) => {
                            tracing::info!(
                                "Received unsubscribe message: {:?}",
                                payload
                            );
                            let SubscriptionType::Stream(subject_wildcard) =
                                payload.topic;

                            if let Err(e) =
                                verify_subject_name(&subject_wildcard)
                            {
                                close_socket_with_error(e, session).await;
                                return;
                            }
                        }
                    }
                }
                Message::Close(reason) => {
                    tracing::info!(
                        "Got close event, terminating session with reason {:?}",
                        reason
                    );
                    let _ = session.close(reason).await;
                    return;
                }
                _ => {
                    tracing::error!("Received unknown message type");
                    let _ = session.close(None).await;
                    return;
                }
            };
        }
    });

    Ok(response)
}

fn parse_client_message(
    msg: Bytes,
) -> Result<ClientMessage, WsSubscriptionError> {
    let msg = serde_json::from_slice::<ClientMessage>(&msg)
        .map_err(WsSubscriptionError::UnparsablePayload)?;
    Ok(msg)
}

fn stream_to_server_message<S: Streamable>(
    msg: StreamData<S>,
) -> Result<Vec<u8>, WsSubscriptionError> {
    let serialized_data = serde_json::to_vec::<StreamData<S>>(&msg)
        .map_err(WsSubscriptionError::UnserializableMessagePayload)?;
    let server_message =
        serde_json::to_vec(&ServerMessage::Update(serialized_data))
            .map_err(WsSubscriptionError::UnserializableMessagePayload)?;
    Ok(server_message)
}

fn verify_subject_name(
    subject_wildcard: &str,
) -> Result<String, WsSubscriptionError> {
    let mut subject_parts = subject_wildcard.split('.');
    // TODO: more advanced checks here with Regex
    if subject_parts.clone().count() == 1 {
        return Err(WsSubscriptionError::UnsupportedWildcardPattern(
            subject_wildcard.to_string(),
        ));
    }
    let subject_name = subject_parts.next().unwrap_or_default();
    if !Streams::is_within_subject_names(subject_name) {
        return Err(WsSubscriptionError::UnknownSubjectName(
            subject_wildcard.to_string(),
        ));
    }
    Ok(subject_name.to_string())
}

async fn close_socket_with_error(e: WsSubscriptionError, mut session: Session) {
    tracing::error!("Ws subscription error: {:?}", e.to_string());
    let err = serde_json::to_vec(&ServerMessage::Error(e.to_string()))
        .ok()
        .unwrap_or_default();
    let _ = session.binary(err).await;
    let _ = session.close(None).await;
}
