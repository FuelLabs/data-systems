use std::sync::Arc;

use actix_ws::Session;
use uuid::Uuid;

use super::models::SubscriptionPayload;
use crate::telemetry::Telemetry;

/// Represents the context for a WebSocket connection
#[derive(Clone)]
pub struct WsContext {
    pub user_id: Uuid,
    pub session: Session,
    pub telemetry: Arc<Telemetry>,
    pub payload: Option<SubscriptionPayload>,
}

impl WsContext {
    /// Creates a new WebSocket context
    pub fn new(
        user_id: Uuid,
        session: Session,
        telemetry: Arc<Telemetry>,
    ) -> Self {
        Self {
            user_id,
            session,
            telemetry,
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
    pub async fn close_with_error(
        mut self,
        error: super::errors::WsSubscriptionError,
    ) {
        tracing::error!("ws subscription error: {:?}", error.to_string());
        if let Some(payload) = self.payload {
            let payload_str = payload.to_string();
            self.telemetry
                .update_error_metrics(&payload_str, &error.to_string());
            self.telemetry
                .update_unsubscribed(self.user_id, &payload_str);
        }

        self.telemetry.decrement_subscriptions_count();
        super::socket::send_message_to_socket(
            &mut self.session,
            super::models::ServerMessage::Error(error.to_string()),
        )
        .await;

        let _ = self.session.close(None).await;
    }
}
