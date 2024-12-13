use std::sync::{atomic::AtomicUsize, Arc};

use actix_web::{
    web::{self, Bytes},
    HttpMessage,
    HttpRequest,
    Responder,
};
use actix_ws::{Message, Session};
use fuel_streams::{
    logs::Log,
    types::{Block, Input, Output, Receipt, Transaction},
    utxos::Utxo,
    StreamEncoder,
    Streamable,
};
use fuel_streams_core::SubscriptionConfig;
use fuel_streams_storage::DeliverPolicy;
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
            models::{ServerMessage, SubscriptionPayload, SubscriptionType},
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
                                match verify_and_extract_subject_name(
                                    &subject_wildcard,
                                ) {
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

                            // reply to socket with subscription
                            send_message_to_socket(
                                &mut session,
                                ServerMessage::Subscribed(
                                    SubscriptionPayload {
                                        topic: SubscriptionType::Stream(
                                            subject_wildcard.clone(),
                                        ),
                                        from: None,
                                        to: None,
                                    },
                                ),
                            )
                            .await;

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
                                let config = SubscriptionConfig {
                                    deliver_policy: DeliverPolicy::All,
                                    filter_subjects: vec![
                                        subject_wildcard.clone()
                                    ],
                                };
                                let mut sub = match streams
                                    .subscribe(&sub_subject, Some(config))
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
                                while let Some(s3_serialized_payload) =
                                    sub.next().await
                                {
                                    // decode and serialize back to ws payload
                                    let serialized_ws_payload = match decode(
                                        &subject_wildcard,
                                        s3_serialized_payload,
                                    )
                                    .await
                                    {
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

                                    // send the payload over the stream
                                    let _ = stream_session
                                        .binary(serialized_ws_payload)
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

                            if let Err(e) = verify_and_extract_subject_name(
                                &subject_wildcard,
                            ) {
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

                            // send a message to the client to confirm unsubscribing
                            send_message_to_socket(
                                &mut session,
                                ServerMessage::Unsubscribed(
                                    SubscriptionPayload {
                                        topic: SubscriptionType::Stream(
                                            subject_wildcard,
                                        ),
                                        from: None,
                                        to: None,
                                    },
                                ),
                            )
                            .await;
                            return;
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

pub fn verify_and_extract_subject_name(
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
    send_message_to_socket(&mut session, ServerMessage::Error(e.to_string()))
        .await;
    let _ = session.close(None).await;
}

async fn send_message_to_socket(session: &mut Session, message: ServerMessage) {
    let data = serde_json::to_vec(&message).ok().unwrap_or_default();
    let _ = session.binary(data).await;
}

async fn decode(
    subject_wildcard: &str,
    s3_payload: Vec<u8>,
) -> Result<Vec<u8>, WsSubscriptionError> {
    let subject = verify_and_extract_subject_name(subject_wildcard)?;
    match subject.as_str() {
        Transaction::NAME => {
            let entity = Transaction::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        Block::NAME => {
            let entity = Block::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        Input::NAME => {
            let entity = Input::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        Output::NAME => {
            let entity = Output::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        Receipt::NAME => {
            let entity = Receipt::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        Utxo::NAME => {
            let entity = Utxo::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        Log::NAME => {
            let entity = Log::decode_or_panic(s3_payload);
            let serialized_data = serde_json::to_vec(&entity)
                .map_err(WsSubscriptionError::UnparsablePayload)?;
            stream_to_server_message(serialized_data)
        }
        _ => Err(WsSubscriptionError::UnknownSubjectName(subject.to_string())),
    }
}
