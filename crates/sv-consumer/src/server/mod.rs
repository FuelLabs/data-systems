pub(crate) mod metrics;
pub(crate) mod state;

use std::sync::Arc;

use fuel_message_broker::MessageBroker;
use fuel_web_utils::{
    server::api::build_and_spawn_web_server,
    telemetry::Telemetry,
};
use metrics::Metrics;

use crate::{errors::ConsumerError, state::ServerState};

pub struct Server {
    port: u16,
    message_broker: Arc<dyn MessageBroker>,
    telemetry: Arc<Telemetry<Metrics>>,
}

impl Server {
    pub fn new(
        port: u16,
        message_broker: Arc<dyn MessageBroker>,
        telemetry: Arc<Telemetry<Metrics>>,
    ) -> Self {
        Self {
            port,
            message_broker,
            telemetry,
        }
    }

    pub async fn start(self) -> Result<(), ConsumerError> {
        let server_state =
            ServerState::new(self.message_broker, Arc::clone(&self.telemetry));

        build_and_spawn_web_server(self.port, server_state)
            .await
            .map_err(|_| ConsumerError::WebServerStart)?;
        Ok(())
    }
}
