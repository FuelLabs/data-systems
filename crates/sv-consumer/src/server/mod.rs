pub(crate) mod metrics;
pub(crate) mod state;

use std::sync::Arc;

use fuel_message_broker::MessageBroker;
use fuel_web_utils::{
    server::api::build_and_spawn_web_server,
    telemetry::Telemetry,
};

use crate::{errors::ConsumerError, state::ServerState};

pub struct Server {
    port: u16,
    message_broker: Arc<dyn MessageBroker>,
}

impl Server {
    pub fn new(port: u16, message_broker: Arc<dyn MessageBroker>) -> Self {
        Self {
            port,
            message_broker,
        }
    }

    pub async fn start(self) -> Result<(), ConsumerError> {
        let telemetry = Telemetry::new(None)
            .await
            .map_err(|_| ConsumerError::TelemetryStart)?;

        telemetry
            .start()
            .await
            .map_err(|_| ConsumerError::TelemetryStart)?;

        let server_state =
            ServerState::new(self.message_broker, Arc::clone(&telemetry));

        build_and_spawn_web_server(self.port, server_state)
            .await
            .map_err(|_| ConsumerError::WebServerStart)?;
        Ok(())
    }
}
