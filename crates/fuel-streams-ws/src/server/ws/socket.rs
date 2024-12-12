use std::sync::{atomic::AtomicUsize, Arc};

use actix_web::{
    web::{self, Bytes},
    HttpMessage,
    HttpRequest,
    Responder,
};
use actix_ws::{Message, Session};
use futures::StreamExt;
use uuid::Uuid;

use super::{
    errors::WsSubscriptionError,
    fuel_streams::FuelStreams,
    models::ClientMessage,
};
use crate::{
    server::{
        state::ServerState,
        ws::{
            fuel_streams::FuelStreamsExt,
            models::{ServerMessage, SubscriptionType},
        },
    },
    telemetry::Telemetry,
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

    // record the new subscription
    state.context.telemetry.increment_subscriptions_count();

    // spawm an actor handling the ws connection
    let streams = state.context.fuel_streams.clone();
    let telemetry = state.context.telemetry.clone();
    actix_web::rt::spawn(async move {
        tracing::info!("Ws opened for user id {:?}", user_id.to_string());
        while let Some(Ok(msg)) = msg_stream.recv().await {
            let mut session = session.clone();
            match msg {
                Message::Ping(bytes) => {
                    tracing::info!("Received ping, {:?}", bytes);
                    if session.pong(&bytes).await.is_err() {
                        tracing::error!("Error sending pong, {:?}", bytes);
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
                            close_socket_with_error(
                                e, user_id, session, None, telemetry,
                            )
                            .await;
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
                            let sub_subject =
                                match verify_subject_name(&subject_wildcard) {
                                    Ok(res) => res,
                                    Err(e) => {
                                        close_socket_with_error(
                                            e,
                                            user_id,
                                            session,
                                            Some(subject_wildcard.clone()),
                                            telemetry,
                                        )
                                        .await;
                                        return;
                                    }
                                };

                            // start the streamer async
                            let mut stream_session = session.clone();

                            // receive streaming in a background thread
                            let streams = streams.clone();
                            let telemetry = telemetry.clone();
                            actix_web::rt::spawn(async move {
                                // update metrics
                                telemetry.update_user_subscription_metrics(
                                    user_id,
                                    &subject_wildcard,
                                );

                                // subscribe to the stream
                                let mut sub = match streams
                                    .subscribe(&sub_subject, None)
                                    .await
                                {
                                    Ok(sub) => sub,
                                    Err(e) => {
                                        close_socket_with_error(
                                            WsSubscriptionError::Stream(e),
                                            user_id,
                                            session,
                                            Some(subject_wildcard.clone()),
                                            telemetry,
                                        )
                                        .await;
                                        return;
                                    }
                                };

                                // consume and forward to the ws
                                while let Some(res) = sub.next().await {
                                    let serialized_payload =
                                        match stream_to_server_message(res) {
                                            Ok(res) => res,
                                            Err(e) => {
                                                telemetry.update_error_metrics(
                                                    &subject_wildcard,
                                                    &e.to_string(),
                                                );
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
                                close_socket_with_error(
                                    e,
                                    user_id,
                                    session,
                                    Some(subject_wildcard.clone()),
                                    telemetry,
                                )
                                .await;
                                return;
                            }

                            // TODO: implement unsubscribe and session management
                        }
                    }
                }
                Message::Close(reason) => {
                    tracing::info!(
                        "Got close event, terminating session with reason {:?}",
                        reason
                    );
                    let reason_str =
                        reason.and_then(|r| r.description).unwrap_or_default();
                    close_socket_with_error(
                        WsSubscriptionError::ClosedWithReason(
                            reason_str.to_string(),
                        ),
                        user_id,
                        session,
                        None,
                        telemetry,
                    )
                    .await;
                    return;
                }
                _ => {
                    tracing::error!("Received unknown message type");
                    close_socket_with_error(
                        WsSubscriptionError::ClosedWithReason(
                            "Unknown message type".to_string(),
                        ),
                        user_id,
                        session,
                        None,
                        telemetry,
                    )
                    .await;
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

fn stream_to_server_message(
    msg: Vec<u8>,
) -> Result<Vec<u8>, WsSubscriptionError> {
    let server_message = serde_json::to_vec(&ServerMessage::Update(msg))
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
    if !FuelStreams::is_within_subject_names(subject_name) {
        return Err(WsSubscriptionError::UnknownSubjectName(
            subject_wildcard.to_string(),
        ));
    }
    Ok(subject_name.to_string())
}

async fn close_socket_with_error(
    e: WsSubscriptionError,
    user_id: uuid::Uuid,
    mut session: Session,
    subject_wildcard: Option<String>,
    telemetry: Arc<Telemetry>,
) {
    tracing::error!("ws subscription error: {:?}", e.to_string());
    if let Some(subject_wildcard) = subject_wildcard {
        telemetry.update_error_metrics(&subject_wildcard, &e.to_string());
        telemetry.update_unsubscribed(user_id, &subject_wildcard);
    }
    telemetry.decrement_subscriptions_count();
    let err = serde_json::to_vec(&ServerMessage::Error(e.to_string()))
        .ok()
        .unwrap_or_default();
    let _ = session.binary(err).await;
    let _ = session.close(None).await;
}
