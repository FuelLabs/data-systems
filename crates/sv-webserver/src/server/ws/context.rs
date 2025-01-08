use std::sync::Arc;

use actix_ws::Session;
use uuid::Uuid;

use crate::telemetry::Telemetry;

/// Represents the context for a WebSocket connection
#[derive(Clone)]
pub struct WsContext {
    pub user_id: Uuid,
    pub session: Session,
    pub telemetry: Arc<Telemetry>,
    pub subject_wildcard: Option<String>,
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
            subject_wildcard: None,
        }
    }

    /// Creates a new context with a subject wildcard
    pub fn with_subject_wildcard(self, subject_wildcard: &str) -> Self {
        Self {
            subject_wildcard: Some(subject_wildcard.to_string()),
            ..self
        }
    }

    /// Updates error metrics and closes the socket with an error message
    pub async fn close_with_error(
        mut self,
        error: super::errors::WsSubscriptionError,
    ) {
        tracing::error!("ws subscription error: {:?}", error.to_string());
        if let Some(subject_wildcard) = self.subject_wildcard {
            self.telemetry
                .update_error_metrics(&subject_wildcard, &error.to_string());
            self.telemetry
                .update_unsubscribed(self.user_id, &subject_wildcard);
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
