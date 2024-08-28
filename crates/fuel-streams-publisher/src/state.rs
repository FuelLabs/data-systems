use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use fuel_core_bin::FuelService;
use tokio::sync::Mutex;

use crate::Publisher;

#[derive(Clone)]
pub struct SharedState {
    pub fuel_service: Arc<FuelService>,
}

impl SharedState {
    pub async fn health_check(&self) -> impl Responder {
        // TODO: use fuel service to check status of services
        HttpResponse::Ok().body("OK")
    }

    pub async fn metrics(&self) -> impl Responder {
        // TODO: use fuel service to check status of services
        HttpResponse::Ok().body("OK")
    }
}
