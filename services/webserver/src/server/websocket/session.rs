use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use fuel_streams_core::{
    server::{ServerResponse, Subscription},
    FuelStreams,
};
use fuel_web_utils::{
    api_key::{rate_limiter::RateLimitsController, ApiKey},
    telemetry::Telemetry,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
    StreamExt,
};
use tokio::sync::{watch, Mutex};

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
        sender: &mut SplitSink<WebSocket, Message>,
        message: ServerResponse,
    ) -> Result<(), WebsocketError> {
        let msg_encoded =
            serde_json::to_vec(&message).map_err(WebsocketError::Serde)?;
        let msg_encoded = axum::body::Bytes::from(msg_encoded);
        sender.send(Message::Binary(msg_encoded)).await?;
        Ok(())
    }

    async fn send_error(
        &self,
        sender: &mut SplitSink<WebSocket, Message>,
        error: &WebsocketError,
    ) -> Result<(), WebsocketError> {
        let api_key = self.api_key.to_owned();
        let error_msg = ServerResponse::Error(error.to_string());
        if let Err(send_err) = self.send_message(sender, error_msg).await {
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
}

impl MetricsHandler {
    fn new(telemetry: Arc<Telemetry<Metrics>>) -> Self {
        Self { telemetry }
    }

    fn track_subscription(&self, change: SubscriptionChange) {
        if let Some(metrics) = self.telemetry.base_metrics() {
            metrics.update_user_subscription_count(&change);
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
            metrics.track_connection_duration(duration);
        }
    }
}

#[derive(Clone)]
pub struct SubscriptionManager {
    api_key: ApiKey,
    start_time: Instant,
    sender: watch::Sender<bool>,
    pub active_subscriptions: Arc<DashMap<Subscription, ()>>,
    metrics_handler: MetricsHandler,
    rate_limiter: Arc<RateLimitsController>,
}

impl SubscriptionManager {
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
        let api_key = self.api_key.clone();
        self.active_subscriptions.insert(subscription.clone(), ());
        self.rate_limiter.add_active_key_sub(api_key.id());
        self.metrics_handler
            .track_subscription(SubscriptionChange::Added);

        let api_key_id = api_key.id();
        let api_key_role = api_key.role();
        self.rate_limiter
            .check_subscriptions(api_key_id, api_key_role)?;

        Ok(())
    }

    async fn remove_subscription(&self, subscription: &Subscription) {
        tracing::info!("Removing subscription: {:?}", subscription);
        self.shutdown().await;
        if self.active_subscriptions.remove(subscription).is_some() {
            self.metrics_handler
                .track_subscription(SubscriptionChange::Removed);
        }
        self.rate_limiter.remove_active_key_sub(self.api_key.id());
    }

    pub async fn clear_subscriptions(&self) {
        for _entry in self.active_subscriptions.iter() {
            self.metrics_handler
                .track_subscription(SubscriptionChange::Removed);
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
    pub subscription_manager: SubscriptionManager,
    pub streams: Arc<FuelStreams>,
    pub socket_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub socket_receiver: Arc<Mutex<SplitStream<WebSocket>>>,
}

impl WsSession {
    pub fn new(
        api_key: &ApiKey,
        telemetry: Arc<Telemetry<Metrics>>,
        streams: Arc<FuelStreams>,
        rate_limiter: Arc<RateLimitsController>,
        socket: WebSocket,
    ) -> Self {
        let metrics = MetricsHandler::new(telemetry);
        let connection =
            SubscriptionManager::new(api_key, metrics, rate_limiter);
        let messaging = MessageHandler::new(api_key);
        let (sender, receiver) = socket.split();
        Self {
            api_key: api_key.to_owned(),
            messaging,
            subscription_manager: connection,
            streams,
            socket_sender: Arc::new(Mutex::new(sender)),
            socket_receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn receiver(&self) -> watch::Receiver<bool> {
        self.subscription_manager.subscribe()
    }

    pub async fn shutdown(&self) {
        self.subscription_manager.shutdown().await;
    }

    pub fn api_key(&self) -> &ApiKey {
        &self.api_key
    }

    pub async fn send_message(
        &self,
        message: ServerResponse,
    ) -> Result<(), WebsocketError> {
        {
            let mut sender = self.socket_sender.lock().await;
            self.messaging.send_message(&mut sender, message).await?;
        }
        Ok(())
    }

    pub async fn send_socket_message(
        &self,
        message: Message,
    ) -> Result<(), WebsocketError> {
        {
            let mut sender = self.socket_sender.lock().await;
            sender.send(message).await?;
        }
        Ok(())
    }

    pub async fn send_error_msg(
        &self,
        error: &WebsocketError,
    ) -> Result<(), WebsocketError> {
        {
            let mut sender = self.socket_sender.lock().await;
            self.messaging.send_error(&mut sender, error).await?;
        }
        Ok(())
    }

    pub async fn is_subscribed(&self, subscription: &Subscription) -> bool {
        self.subscription_manager.is_subscribed(subscription).await
    }

    pub async fn add_subscription(
        &self,
        subscription: &Subscription,
    ) -> Result<(), WebsocketError> {
        self.subscription_manager
            .add_subscription(subscription)
            .await
    }

    pub async fn remove_subscription(&self, subscription: &Subscription) {
        self.subscription_manager
            .remove_subscription(subscription)
            .await;
    }

    pub async fn close_session(
        &self,
        action: &CloseAction,
    ) -> Result<(), WebsocketError> {
        self.shutdown().await;
        if let CloseAction::Error(err) = action {
            self.send_error_msg(err).await?;
        }
        self.subscription_manager.clear_subscriptions().await;
        let duration = self.subscription_manager.connection_duration();
        self.subscription_manager
            .metrics_handler
            .track_connection_duration(duration);
        self.log_connection_close(duration, action);
        Ok(())
    }

    fn log_connection_close(&self, duration: Duration, action: &CloseAction) {
        let api_key = self.api_key();
        let close_frame: Option<axum::extract::ws::CloseFrame> = action.into();
        let (code, description) = match close_frame {
            Some(frame) => (frame.code, Some(frame.reason.to_string())),
            None => (axum::extract::ws::close_code::NORMAL, None),
        };
        if code == axum::extract::ws::close_code::NORMAL {
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
        SubscriptionManager::MAX_FRAME_SIZE
    }
}
