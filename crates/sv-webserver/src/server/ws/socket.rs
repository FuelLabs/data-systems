use std::{
    str::FromStr,
    sync::{atomic::AtomicUsize, Arc},
};

use actix_web::{
    web::{self, Bytes},
    HttpMessage,
    HttpRequest,
    Responder,
};
use actix_ws::{Message, Session};
use fuel_streams_core::{nats::*, stream::*, types::*};
use fuel_streams_store::{
    db::Db,
    record::{DataEncoder, RecordEntity},
};
use futures::StreamExt;
use uuid::Uuid;

use super::{
    errors::WsSubscriptionError,
    models::{ClientMessage, ResponseMessage},
};
use crate::{
    server::{
        state::ServerState,
        ws::models::{ServerMessage, SubscriptionPayload},
    },
    telemetry::Telemetry,
};

static _NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct WsContext {
    user_id: uuid::Uuid,
    session: Session,
    telemetry: Arc<Telemetry>,
    subject_wildcard: Option<String>,
}
impl WsContext {
    fn with_subject_wildcard(self, subject_wildcard: &str) -> Self {
        Self {
            subject_wildcard: Some(subject_wildcard.to_string()),
            ..self
        }
    }
}

/// Macro to handle WebSocket errors using WsContext
macro_rules! handle_ws_error {
    ($result:expr, $ctx:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                close_socket_with_error(
                    e.into(),
                    $ctx.user_id,
                    $ctx.session,
                    $ctx.subject_wildcard.clone(),
                    $ctx.telemetry,
                )
                .await;
                return Ok(());
            }
        }
    };
}

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
                    let bytes = Bytes::from(string.as_bytes().to_vec());
                    let _ = handle_binary_message(
                        bytes,
                        user_id,
                        session,
                        Arc::clone(&telemetry),
                        Arc::clone(&streams),
                    )
                    .await;
                }
                Message::Binary(bytes) => {
                    let _ = handle_binary_message(
                        bytes,
                        user_id,
                        session,
                        Arc::clone(&telemetry),
                        Arc::clone(&streams),
                    )
                    .await;
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

async fn handle_binary_message(
    msg: Bytes,
    user_id: uuid::Uuid,
    mut session: Session,
    telemetry: Arc<Telemetry>,
    streams: Arc<FuelStreams>,
) -> Result<(), WsSubscriptionError> {
    tracing::info!("Received binary {:?}", msg);

    let ctx = WsContext {
        user_id,
        telemetry: Arc::clone(&telemetry),
        subject_wildcard: None,
        session: session.clone(),
    };

    let client_message = handle_ws_error!(parse_client_message(msg), ctx);
    match client_message {
        ClientMessage::Subscribe(payload) => {
            tracing::info!("Received subscribe message: {:?}", payload);
            let subject_wildcard = payload.wildcard.clone();
            let deliver_policy = payload.deliver_policy;

            // verify the subject name
            let entity = handle_ws_error!(
                verify_and_extract_subject_name(&subject_wildcard),
                ctx.clone().with_subject_wildcard(&subject_wildcard)
            );

            let record_entity = RecordEntity::from_str(&entity).unwrap();
            let nats_client = streams.nats_client();
            let db = streams.db.clone();
            let mut sub = handle_ws_error!(
                create_subscriber(
                    &record_entity,
                    &nats_client,
                    &db,
                    subject_wildcard.clone()
                )
                .await,
                ctx
            );

            let mut stream_session = session.clone();
            send_message_to_socket(
                &mut session,
                ServerMessage::Subscribed(SubscriptionPayload {
                    wildcard: subject_wildcard.clone(),
                    deliver_policy,
                }),
            )
            .await;

            // receive streaming in a background thread
            let telemetry = telemetry.clone();
            let subject_wildcard = subject_wildcard.clone();
            let record_entity = record_entity.clone();
            actix_web::rt::spawn(async move {
                telemetry.update_user_subscription_metrics(
                    user_id,
                    &subject_wildcard,
                );

                // consume and forward to the ws
                while let Some(message) = sub.next().await {
                    let serialized_ws_payload = match decode(
                        &record_entity,
                        &message.subject,
                        message.payload,
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
                    let _ = stream_session.binary(serialized_ws_payload).await;
                    // let _ = message.ack().await;
                }
            });
            Ok(())
        }
        ClientMessage::Unsubscribe(payload) => {
            tracing::info!("Received unsubscribe message: {:?}", payload);
            let subject_wildcard = payload.wildcard.clone();
            let deliver_policy = payload.deliver_policy;

            handle_ws_error!(
                verify_and_extract_subject_name(&subject_wildcard),
                ctx.clone().with_subject_wildcard(&subject_wildcard)
            );

            send_message_to_socket(
                &mut session,
                ServerMessage::Unsubscribed(SubscriptionPayload {
                    wildcard: subject_wildcard,
                    deliver_policy,
                }),
            )
            .await;
            Ok(())
        }
    }
}

fn parse_client_message(
    msg: Bytes,
) -> Result<ClientMessage, WsSubscriptionError> {
    let msg = serde_json::from_slice::<ClientMessage>(&msg)
        .map_err(WsSubscriptionError::UnserializablePayload)?;
    Ok(msg)
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

async fn decode<'a>(
    stream_type: &'a RecordEntity,
    subject: &str,
    payload: Bytes,
) -> Result<Vec<u8>, WsSubscriptionError> {
    let subject = verify_and_extract_subject_name(subject)?;
    let json_value = match stream_type {
        RecordEntity::Block => {
            Block::decode(&payload).await?.to_json_value()?
        }
        RecordEntity::Transaction => {
            Transaction::decode(&payload).await?.to_json_value()?
        }
        RecordEntity::Input => {
            Input::decode(&payload).await?.to_json_value()?
        }
        RecordEntity::Output => {
            Output::decode(&payload).await?.to_json_value()?
        }
        RecordEntity::Receipt => {
            Receipt::decode(&payload).await?.to_json_value()?
        }
        RecordEntity::Utxo => Utxo::decode(&payload).await?.to_json_value()?,
        RecordEntity::Log => Log::decode(&payload).await?.to_json_value()?,
    };

    serde_json::to_vec(&ServerMessage::Response(ResponseMessage {
        subject: subject.to_string(),
        payload: json_value,
    }))
    .map_err(WsSubscriptionError::UnserializablePayload)
}

async fn create_subscriber(
    record_entity: &RecordEntity,
    nats_client: &Arc<NatsClient>,
    db: &Arc<Db>,
    subject_wildcard: String,
) -> Result<StreamLiveSubscriber, StreamError> {
    let streams = FuelStreams::new(nats_client, db).await;
    match record_entity {
        RecordEntity::Block => {
            streams.blocks.subscribe_live(subject_wildcard).await
        }
        RecordEntity::Transaction => {
            streams.transactions.subscribe_live(subject_wildcard).await
        }
        RecordEntity::Input => {
            streams.inputs.subscribe_live(subject_wildcard).await
        }
        RecordEntity::Output => {
            streams.outputs.subscribe_live(subject_wildcard).await
        }
        RecordEntity::Receipt => {
            streams.receipts.subscribe_live(subject_wildcard).await
        }
        RecordEntity::Utxo => {
            streams.utxos.subscribe_live(subject_wildcard).await
        }
        RecordEntity::Log => {
            streams.logs.subscribe_live(subject_wildcard).await
        }
    }
}
