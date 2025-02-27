use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use actix_ws::{CloseCode, CloseReason, Session};
use fuel_streams_core::{
    server::{ServerResponse, Subscription},
    FuelStreams,
};
use fuel_web_utils::{
    api_key::{rate_limiter::RateLimitsController, ApiKey},
    telemetry::Telemetry,
};
use tokio::sync::{broadcast, Mutex};

use crate::{
    metrics::{Metrics, SubscriptionChange},
    server::{errors::WebsocketError, handlers::websocket::CloseAction},
};

#[derive(Clone)]
struct MessageHandler {
    api_key: ApiKey,
}

impl MessageHandler {
    fn new(api_key: &ApiKey) -> Self {
        Self {
            api_key: api_key.to_owned(),
        }
    }

    async fn send_message(
        &self,
        session: &mut Session,
        message: ServerResponse,
    ) -> Result<(), WebsocketError> {
        let msg_encoded =
            serde_json::to_vec(&message).map_err(WebsocketError::Serde)?;
        session.binary(msg_encoded).await?;
        Ok(())
    }

    async fn send_error(
        &self,
        session: &mut Session,
        error: &WebsocketError,
    ) -> Result<(), WebsocketError> {
        let api_key = self.api_key.to_owned();
        let error_msg = ServerResponse::Error(error.to_string());
        if let Err(send_err) = self.send_message(session, error_msg).await {
            tracing::error!(
                %api_key,
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
    api_key: ApiKey,
}

impl MetricsHandler {
    fn new(telemetry: Arc<Telemetry<Metrics>>, api_key: &ApiKey) -> Self {
        Self {
            telemetry,
            api_key: api_key.to_owned(),
        }
    }

    fn track_subscription(
        &self,
        subscription: &Subscription,
        change: SubscriptionChange,
    ) {
        if let Some(metrics) = self.telemetry.base_metrics() {
            let subject = subscription.payload.subject.clone();
            metrics.update_user_subscription_count(
                self.api_key.id(),
                self.api_key.user(),
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
            metrics.track_connection_duration(
                self.api_key.id(),
                self.api_key.user(),
                duration,
            );
        }
    }

    fn track_duplicate_subscription(&self, subscription: &Subscription) {
        if let Some(metrics) = self.telemetry.base_metrics() {
            metrics.track_duplicate_subscription(
                self.api_key.id(),
                self.api_key.user(),
                subscription,
            );
        }
    }
}

// Connection management
#[derive(Clone)]
pub struct ConnectionManager {
    api_key: ApiKey,
    start_time: Instant,
    tx: broadcast::Sender<ApiKey>,
    active_subscriptions: Arc<Mutex<HashSet<Subscription>>>,
    metrics_handler: MetricsHandler,
    rate_limiter: Arc<RateLimitsController>,
}

impl ConnectionManager {
    pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
    pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    pub const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024; // 8MB
    pub const CHANNEL_CAPACITY: usize = 100;

    fn new(
        api_key: &ApiKey,
        metrics_handler: MetricsHandler,
        rate_limiter: Arc<RateLimitsController>,
    ) -> Self {
        let (tx, _) = broadcast::channel(Self::CHANNEL_CAPACITY);
        Self {
            api_key: api_key.to_owned(),
            start_time: Instant::now(),
            tx,
            active_subscriptions: Arc::new(Mutex::new(HashSet::new())),
            metrics_handler,
            rate_limiter,
        }
    }

    fn subscribe(&self) -> broadcast::Receiver<ApiKey> {
        self.tx.subscribe()
    }

    async fn shutdown(&self, api_key: &ApiKey) {
        let _ = self.tx.send(api_key.to_owned());
    }

    async fn is_subscribed(&self, subscription: &Subscription) -> bool {
        self.active_subscriptions
            .lock()
            .await
            .contains(subscription)
    }

    async fn add_subscription(
        &self,
        subscription: &Subscription,
    ) -> Result<(), WebsocketError> {
        self.active_subscriptions
            .lock()
            .await
            .insert(subscription.to_owned());

        self.rate_limiter.add_active_key_sub(self.api_key.id());

        self.metrics_handler
            .track_subscription(subscription, SubscriptionChange::Added);
        Ok(())
    }

    async fn remove_subscription(&self, subscription: &Subscription) {
        tracing::info!("Removing subscription: {:?}", subscription);
        self.shutdown(&self.api_key).await;
        if self.active_subscriptions.lock().await.remove(subscription) {
            self.metrics_handler
                .track_subscription(subscription, SubscriptionChange::Removed);
        }
        self.rate_limiter.remove_active_key_sub(self.api_key.id());
    }

    pub async fn clear_subscriptions(&self) {
        let subscriptions = self.active_subscriptions.lock().await;
        for item in subscriptions.iter() {
            self.remove_subscription(item).await;
            self.metrics_handler
                .track_subscription(item, SubscriptionChange::Removed);
        }
    }

    fn connection_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    async fn check_duplicate_subscription(
        &self,
        session: &mut Session,
        subscription: &Subscription,
        message_handler: &MessageHandler,
    ) -> Result<bool, WebsocketError> {
        if self.is_subscribed(subscription).await {
            self.metrics_handler
                .track_duplicate_subscription(subscription);
            let warning_msg = ServerResponse::Error(format!(
                "Already subscribed to {}",
                subscription
            ));
            message_handler.send_message(session, warning_msg).await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn heartbeat(
        &self,
        api_key: &ApiKey,
        session: &mut Session,
        last_heartbeat: Instant,
    ) -> Result<(), WebsocketError> {
        let duration = Instant::now().duration_since(last_heartbeat);
        if duration > Self::CLIENT_TIMEOUT {
            tracing::warn!(
                %api_key,
                timeout = ?Self::CLIENT_TIMEOUT,
                "Client timeout; disconnecting"
            );
            return Err(WebsocketError::Timeout);
        }
        session.ping(b"").await.map_err(WebsocketError::from)
    }
}

#[derive(Clone)]
pub struct WsSession {
    api_key: ApiKey,
    messaging: MessageHandler,
    pub connection: ConnectionManager,
    pub streams: Arc<FuelStreams>,
}

impl WsSession {
    pub fn new(
        api_key: &ApiKey,
        telemetry: Arc<Telemetry<Metrics>>,
        streams: Arc<FuelStreams>,
        rate_limiter: Arc<RateLimitsController>,
    ) -> Self {
        let metrics = MetricsHandler::new(telemetry, api_key);
        let connection = ConnectionManager::new(api_key, metrics, rate_limiter);
        let messaging = MessageHandler::new(api_key);
        Self {
            api_key: api_key.to_owned(),
            messaging,
            connection,
            streams,
        }
    }

    pub fn receiver(&self) -> broadcast::Receiver<ApiKey> {
        self.connection.subscribe()
    }

    pub fn api_key(&self) -> &ApiKey {
        &self.api_key
    }

    pub async fn send_message(
        &self,
        session: &mut Session,
        message: ServerResponse,
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

    pub async fn is_subscribed(&self, subscription: &Subscription) -> bool {
        self.connection.is_subscribed(subscription).await
    }

    pub async fn add_subscription(
        &self,
        subscription: &Subscription,
    ) -> Result<(), WebsocketError> {
        self.connection.add_subscription(subscription).await?;
        Ok(())
    }

    pub async fn remove_subscription(&self, subscription: &Subscription) {
        self.connection.remove_subscription(subscription).await;
    }

    pub async fn check_duplicate_subscription(
        &self,
        session: &mut Session,
        subscription: &Subscription,
    ) -> Result<bool, WebsocketError> {
        self.connection
            .check_duplicate_subscription(
                session,
                subscription,
                &self.messaging,
            )
            .await
    }

    pub async fn close_session(self, session: Session, action: &CloseAction) {
        let _ = session.close(Some(action.into())).await;
        self.connection.clear_subscriptions().await;
        let duration = self.connection.connection_duration();
        self.connection
            .metrics_handler
            .track_connection_duration(duration);
        self.log_connection_close(duration, action);
    }

    fn log_connection_close(&self, duration: Duration, action: &CloseAction) {
        let api_key = self.api_key();
        let close_reason: CloseReason = action.into();
        let description = close_reason.description.as_deref();
        if close_reason.code == CloseCode::Normal {
            tracing::info!(
                target: "websocket",
                %api_key,
                event = "websocket_connection_closed",
                duration_secs = duration.as_secs_f64(),
                close_reason = description,
                "WebSocket connection closed"
            );
        } else {
            tracing::error!(
                target: "websocket",
                %api_key,
                event = "websocket_connection_closed",
                duration_secs = duration.as_secs_f64(),
                close_reason = description,
                "WebSocket connection closed"
            );
        }
    }

    pub async fn heartbeat(
        &self,
        session: &mut Session,
        last_heartbeat: Instant,
    ) -> Result<(), WebsocketError> {
        self.connection
            .heartbeat(self.api_key(), session, last_heartbeat)
            .await
    }

    pub fn heartbeat_interval(&self) -> Duration {
        ConnectionManager::HEARTBEAT_INTERVAL
    }

    pub fn max_frame_size(&self) -> usize {
        ConnectionManager::MAX_FRAME_SIZE
    }
}
