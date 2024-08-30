use std::sync::Arc;

use actix_web::{HttpResponse, Responder};
use fuel_core_bin::FuelService;
use fuel_streams_core::nats::{NatsClient, NatsClientOpts};
#[derive(Clone)]
pub struct SharedState {
    pub fuel_service: Arc<FuelService>,
    pub nats_client: NatsClient,
}

impl SharedState {
    pub async fn new(
        fuel_service: Arc<FuelService>,
        nats_url: &str,
    ) -> anyhow::Result<Self> {
        let nats_client_opts = NatsClientOpts::admin_opts(nats_url);
        let nats_client = NatsClient::connect(&nats_client_opts).await?;
        Ok(Self {
            fuel_service,
            nats_client,
        })
    }
}

impl SharedState {
    pub async fn health_check(&self) -> impl Responder {
        let state = self.fuel_service.state();
        if !state.started() {
            return HttpResponse::ServiceUnavailable()
                .body("Service Unavailable");
        }

        let is_nats_client_connected = self.nats_client.is_connected();
        if !is_nats_client_connected {
            return HttpResponse::ServiceUnavailable()
                .body("Service Unavailable");
        }
        HttpResponse::Ok().finish()
    }

    pub async fn metrics(&self) -> impl Responder {
        // TODO: use fuel service to check status of services
        HttpResponse::Ok().finish()
    }
}
