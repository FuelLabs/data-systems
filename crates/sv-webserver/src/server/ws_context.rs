use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest};
use actix_ws::Session;
use fuel_streams_core::FuelStreams;
use fuel_web_utils::telemetry::Telemetry;
use uuid::Uuid;

use crate::{
    metrics::Metrics,
    server::{
        errors::WebsocketError,
        types::{ServerMessage, SubscriptionPayload},
    },
};

/// Represents the context for a WebSocket connection
#[derive(Clone)]
pub struct WsContext {
    pub user_id: Uuid,
    pub session: Session,
    pub telemetry: Arc<Telemetry<Metrics>>,
    pub payload: Option<SubscriptionPayload>,
    pub streams: Arc<FuelStreams>,
}

impl WsContext {
    /// Creates a new WebSocket context
    pub fn new(
        user_id: Uuid,
        session: Session,
        telemetry: Arc<Telemetry<Metrics>>,
        streams: Arc<FuelStreams>,
    ) -> Self {
        Self {
            user_id,
            session,
            telemetry,
            streams,
            payload: None,
        }
    }

    /// Creates a new context with a subject wildcard
    pub fn with_payload(self, payload: &SubscriptionPayload) -> Self {
        Self {
            payload: Some(payload.clone()),
            ..self
        }
    }

    /// Updates error metrics and closes the socket with an error message
    pub async fn close_with_error(mut self, error: WebsocketError) {
        tracing::error!("ws subscription error: {:?}", error.to_string());
        if let Some(payload) = self.payload.as_ref() {
            if let Some(metrics) = self.telemetry.base_metrics() {
                let payload_str = payload.to_string();
                metrics.update_error_metrics(&payload_str, &error.to_string());
                metrics.update_unsubscribed(self.user_id, &payload_str);
            }
        }

        if let Some(metrics) = self.telemetry.base_metrics() {
            metrics.decrement_subscriptions_count();
        }
        let msg = ServerMessage::Error(error.to_string());
        self.send_message_to_socket(msg).await;
        let _ = self.session.close(None).await;
    }

    pub async fn send_message_to_socket(&mut self, message: ServerMessage) {
        let data = serde_json::to_vec(&message).ok().unwrap_or_default();
        let _ = self.session.binary(data).await;
    }

    pub fn user_id_from_req(
        req: &HttpRequest,
    ) -> Result<Uuid, actix_web::Error> {
        match req.extensions().get::<Uuid>() {
            Some(user_id) => {
                tracing::info!(
                    "Authenticated WebSocket connection for user: {:?}",
                    user_id.to_string()
                );
                Ok(user_id.to_owned())
            }
            None => {
                tracing::info!("Unauthenticated WebSocket connection");
                Err(actix_web::error::ErrorUnauthorized(
                    "Missing or invalid JWT",
                ))
            }
        }
    }
}
