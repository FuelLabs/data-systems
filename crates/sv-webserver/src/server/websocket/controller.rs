use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use actix_web::{HttpMessage, HttpRequest};
use actix_ws::{CloseCode, CloseReason, Session};
use fuel_streams_core::FuelStreams;
use fuel_web_utils::telemetry::Telemetry;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use crate::{
    metrics::{Metrics, SubscriptionChange},
    server::{
        errors::WebsocketError,
        types::{ServerMessage, SubscriptionPayload},
    },
};

#[derive(Clone)]
struct AuthManager {
    user_id: Uuid,
}

impl AuthManager {
    fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }

    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub fn user_id_from_req(
        req: &HttpRequest,
    ) -> Result<Uuid, actix_web::Error> {
        match req.extensions().get::<Uuid>() {
            Some(user_id) => {
                tracing::info!(
                    user_id = %user_id,
                    "Authenticated WebSocket connection"
                );
                Ok(*user_id)
            }
            None => {
                tracing::warn!("Unauthenticated WebSocket connection attempt");
                Err(actix_web::error::ErrorUnauthorized(
                    "Missing or invalid JWT",
                ))
            }
        }
    }
}

#[derive(Clone)]
struct MessageHandler {
    user_id: Uuid,
}

impl MessageHandler {
    fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }

    async fn send_message(
        &self,
        session: &mut Session,
        message: ServerMessage,
    ) -> Result<(), WebsocketError> {
        let msg_encoded = serde_json::to_vec(&message)
            .map_err(WebsocketError::UnserializablePayload)?;
        session.binary(msg_encoded).await?;
        Ok(())
    }

    async fn send_error(
        &self,
        session: &mut Session,
        error: &WebsocketError,
    ) -> Result<(), WebsocketError> {
        let error_msg = ServerMessage::Error(error.to_string());
        if let Err(send_err) = self.send_message(session, error_msg).await {
            tracing::error!(
                %self.user_id,
                error = %send_err,
                "Failed to send error message"
            );
            return Err(WebsocketError::SendError);
        }
        Ok(())
    }
}

#[derive(Clone)]
struct MetricsHandler {
    telemetry: Arc<Telemetry<Metrics>>,
    user_id: Uuid,
}

impl MetricsHandler {
    fn new(telemetry: Arc<Telemetry<Metrics>>, user_id: Uuid) -> Self {
        Self { telemetry, user_id }
    }

    fn track_subscription(
        &self,
        payload: &SubscriptionPayload,
        change: SubscriptionChange,
    ) {
        if let Some(metrics) = self.telemetry.base_metrics() {
            let subject = payload.subject.clone();
            metrics.update_user_subscription_count(
                self.user_id,
                &subject,
                &change,
            );
            match change {
                SubscriptionChange::Added => {
                    metrics.increment_subscriptions_count()
                }
                SubscriptionChange::Removed => {
                    metrics.decrement_subscriptions_count()
                }
            }
        }
    }

    fn track_connection_duration(&self, duration: Duration) {
        if let Some(metrics) = self.telemetry.base_metrics() {
            metrics
                .track_connection_duration(&self.user_id.to_string(), duration);
        }
    }

    fn track_duplicate_subscription(&self, payload: &SubscriptionPayload) {
        if let Some(metrics) = self.telemetry.base_metrics() {
            metrics.track_duplicate_subscription(
                self.user_id,
                &payload.to_string(),
            );
        }
    }
}

// Connection management
#[derive(Clone)]
struct ConnectionManager {
    user_id: Uuid,
    start_time: Instant,
    tx: broadcast::Sender<Uuid>,
    active_subscriptions: Arc<Mutex<HashSet<String>>>,
    metrics_handler: MetricsHandler,
}

impl ConnectionManager {
    pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
    pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    pub const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024; // 8MB
    pub const CHANNEL_CAPACITY: usize = 100;

    fn new(user_id: Uuid, metrics_handler: MetricsHandler) -> Self {
        let (tx, _) = broadcast::channel(Self::CHANNEL_CAPACITY);
        Self {
            user_id,
            start_time: Instant::now(),
            tx,
            active_subscriptions: Arc::new(Mutex::new(HashSet::new())),
            metrics_handler,
        }
    }

    fn subscribe(&self) -> broadcast::Receiver<Uuid> {
        self.tx.subscribe()
    }

    async fn is_subscribed(&self, subscription_id: &str) -> bool {
        self.active_subscriptions
            .lock()
            .await
            .contains(subscription_id)
    }

    async fn add_subscription(
        &self,
        payload: &SubscriptionPayload,
    ) -> Result<(), WebsocketError> {
        let subscription_id = payload.to_string();
        self.active_subscriptions
            .lock()
            .await
            .insert(subscription_id);
        self.metrics_handler
            .track_subscription(payload, SubscriptionChange::Added);
        Ok(())
    }

    async fn remove_subscription(&self, payload: &SubscriptionPayload) {
        self.shutdown(self.user_id).await;
        let subscription_id = payload.to_string();
        if self
            .active_subscriptions
            .lock()
            .await
            .remove(&subscription_id)
        {
            self.metrics_handler
                .track_subscription(payload, SubscriptionChange::Removed);
        }
    }

    async fn clear_subscriptions(&self) {
        let subscriptions = self.active_subscriptions.lock().await;
        for subscription_id in subscriptions.iter() {
            let payload =
                SubscriptionPayload::try_from(subscription_id.clone());
            if let Ok(payload) = payload {
                self.remove_subscription(&payload).await;
                self.metrics_handler
                    .track_subscription(&payload, SubscriptionChange::Removed);
            }
        }
    }

    async fn shutdown(&self, user_id: Uuid) {
        let _ = self.tx.send(user_id);
    }

    fn connection_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    async fn check_duplicate_subscription(
        &self,
        session: &mut Session,
        payload: &SubscriptionPayload,
        message_handler: &MessageHandler,
    ) -> Result<bool, WebsocketError> {
        let subscription_id = payload.to_string();
        if self.is_subscribed(&subscription_id).await {
            self.metrics_handler.track_duplicate_subscription(payload);
            let warning_msg = ServerMessage::Error(format!(
                "Already subscribed to {}",
                subscription_id
            ));
            message_handler.send_message(session, warning_msg).await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn heartbeat(
        &self,
        user_id: &Uuid,
        session: &mut Session,
        last_heartbeat: Instant,
    ) -> Result<(), WebsocketError> {
        let duration = Instant::now().duration_since(last_heartbeat);
        if duration > Self::CLIENT_TIMEOUT {
            tracing::warn!(
                %user_id,
                timeout = ?Self::CLIENT_TIMEOUT,
                "Client timeout; disconnecting"
            );
            return Err(WebsocketError::Timeout);
        }
        session.ping(b"").await.map_err(WebsocketError::from)
    }
}

#[derive(Clone)]
pub struct WsController {
    auth: AuthManager,
    messaging: MessageHandler,
    connection: ConnectionManager,
    pub streams: Arc<FuelStreams>,
}

impl WsController {
    pub fn new(
        user_id: Uuid,
        telemetry: Arc<Telemetry<Metrics>>,
        streams: Arc<FuelStreams>,
    ) -> Self {
        let metrics = MetricsHandler::new(telemetry, user_id);
        let connection = ConnectionManager::new(user_id, metrics);
        Self {
            auth: AuthManager::new(user_id),
            messaging: MessageHandler::new(user_id),
            connection,
            streams,
        }
    }

    pub fn receiver(&self) -> broadcast::Receiver<Uuid> {
        self.connection.subscribe()
    }

    pub fn user_id(&self) -> &Uuid {
        self.auth.user_id()
    }

    pub async fn send_message(
        &self,
        session: &mut Session,
        message: ServerMessage,
    ) -> Result<(), WebsocketError> {
        self.messaging.send_message(session, message).await
    }

    pub async fn send_error_msg(
        &self,
        session: &mut Session,
        error: &WebsocketError,
    ) -> Result<(), WebsocketError> {
        self.messaging.send_error(session, error).await
    }

    pub async fn is_subscribed(&self, subscription_id: &str) -> bool {
        self.connection.is_subscribed(subscription_id).await
    }

    pub async fn add_subscription(
        &self,
        payload: &SubscriptionPayload,
    ) -> Result<(), WebsocketError> {
        self.connection.add_subscription(payload).await
    }

    pub async fn remove_subscription(&self, payload: &SubscriptionPayload) {
        self.connection.remove_subscription(payload).await
    }

    pub async fn check_duplicate_subscription(
        &self,
        session: &mut Session,
        payload: &SubscriptionPayload,
    ) -> Result<bool, WebsocketError> {
        self.connection
            .check_duplicate_subscription(session, payload, &self.messaging)
            .await
    }

    pub async fn shutdown_subscription(
        &self,
        session: &mut Session,
        payload: &SubscriptionPayload,
    ) -> Result<(), WebsocketError> {
        let user_id = self.auth.user_id();
        if !self.is_subscribed(&payload.to_string()).await {
            let warning_msg = ServerMessage::Error(format!(
                "No active subscription found for {}",
                payload
            ));
            self.send_message(session, warning_msg).await?;
            return Ok(());
        }
        self.connection.shutdown(user_id.to_owned()).await;
        Ok(())
    }

    pub async fn close_session(
        self,
        session: Session,
        close_reason: CloseReason,
    ) {
        let _ = session.close(Some(close_reason.clone())).await;
        self.connection.clear_subscriptions().await;

        let duration = self.connection.connection_duration();
        self.connection
            .metrics_handler
            .track_connection_duration(duration);

        self.log_connection_close(duration, &close_reason);
    }

    fn log_connection_close(
        &self,
        duration: Duration,
        close_reason: &CloseReason,
    ) {
        let user_id = self.auth.user_id().to_string();
        let description = close_reason.description.as_deref();

        if close_reason.code == CloseCode::Normal {
            tracing::info!(
                target: "websocket",
                %user_id,
                event = "websocket_connection_closed",
                duration_secs = duration.as_secs_f64(),
                close_reason = description,
                "WebSocket connection closed"
            );
        } else {
            tracing::error!(
                target: "websocket",
                %user_id,
                event = "websocket_connection_closed",
                duration_secs = duration.as_secs_f64(),
                close_reason = description,
                "WebSocket connection closed"
            );
        }
    }

    pub fn user_id_from_req(
        req: &HttpRequest,
    ) -> Result<Uuid, actix_web::Error> {
        AuthManager::user_id_from_req(req)
    }

    pub async fn heartbeat(
        &self,
        session: &mut Session,
        last_heartbeat: Instant,
    ) -> Result<(), WebsocketError> {
        self.connection
            .heartbeat(self.user_id(), session, last_heartbeat)
            .await
    }

    pub fn heartbeat_interval(&self) -> Duration {
        ConnectionManager::HEARTBEAT_INTERVAL
    }

    pub fn max_frame_size(&self) -> usize {
        ConnectionManager::MAX_FRAME_SIZE
    }
}
