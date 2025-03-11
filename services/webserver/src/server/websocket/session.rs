use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use actix_ws::{CloseCode, CloseReason, Session};
use dashmap::DashMap;
use fuel_streams_core::{
    server::{ServerResponse, Subscription},
    FuelStreams,
};
use fuel_web_utils::{
    api_key::{rate_limiter::RateLimitsController, ApiKey},
    telemetry::Telemetry,
};
use tokio::sync::watch;

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
}

#[derive(Clone)]
struct ConnectionManager {
    api_key: ApiKey,
    start_time: Instant,
    sender: watch::Sender<bool>,
    active_subscriptions: Arc<DashMap<Subscription, ()>>,
    metrics_handler: MetricsHandler,
    rate_limiter: Arc<RateLimitsController>,
}

impl ConnectionManager {
    pub const MAX_FRAME_SIZE: usize = 8 * 1024 * 1024; // 8MB

    fn new(
        api_key: &ApiKey,
        metrics_handler: MetricsHandler,
        rate_limiter: Arc<RateLimitsController>,
    ) -> Self {
        let (sender, _) = watch::channel(true);
        Self {
            sender,
            api_key: api_key.to_owned(),
            start_time: Instant::now(),
            active_subscriptions: Arc::new(DashMap::new()),
            metrics_handler,
            rate_limiter,
        }
    }

    fn subscribe(&self) -> watch::Receiver<bool> {
        self.sender.subscribe()
    }

    async fn shutdown(&self) {
        let _ = self.sender.send(false);
    }

    async fn is_subscribed(&self, subscription: &Subscription) -> bool {
        self.active_subscriptions.contains_key(subscription)
    }

    async fn add_subscription(
        &self,
        subscription: &Subscription,
    ) -> Result<(), WebsocketError> {
        self.active_subscriptions.insert(subscription.clone(), ());
        self.rate_limiter.add_active_key_sub(self.api_key.id());
        self.metrics_handler
            .track_subscription(subscription, SubscriptionChange::Added);
        Ok(())
    }

    async fn remove_subscription(&self, subscription: &Subscription) {
        tracing::info!("Removing subscription: {:?}", subscription);
        self.shutdown().await;
        if self.active_subscriptions.remove(subscription).is_some() {
            self.metrics_handler
                .track_subscription(subscription, SubscriptionChange::Removed);
        }
        self.rate_limiter.remove_active_key_sub(self.api_key.id());
    }

    pub async fn clear_subscriptions(&self) {
        for entry in self.active_subscriptions.iter() {
            self.metrics_handler
                .track_subscription(entry.key(), SubscriptionChange::Removed);
        }
        self.active_subscriptions.clear();
        self.rate_limiter.remove_active_key_sub(self.api_key.id());
    }

    fn connection_duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[derive(Clone)]
pub struct WsSession {
    api_key: ApiKey,
    messaging: MessageHandler,
    connection: ConnectionManager,
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

    pub fn receiver(&self) -> watch::Receiver<bool> {
        self.connection.subscribe()
    }

    pub async fn shutdown(&self) {
        self.connection.shutdown().await;
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
        self.connection.add_subscription(subscription).await
    }

    pub async fn remove_subscription(&self, subscription: &Subscription) {
        self.connection.remove_subscription(subscription).await;
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

    pub fn max_frame_size(&self) -> usize {
        ConnectionManager::MAX_FRAME_SIZE
    }
}
