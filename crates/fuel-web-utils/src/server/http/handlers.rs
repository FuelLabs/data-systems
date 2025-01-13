use actix_web::{web, HttpResponse, Result};

use crate::server::state::StateProvider;

pub async fn get_metrics<T: StateProvider>(
    state: web::Data<T>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type(
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(state.get_metrics().await))
}

pub async fn get_health<T: StateProvider>(
    state: web::Data<T>,
) -> Result<HttpResponse> {
    if !state.is_healthy().await {
        return Ok(
            HttpResponse::ServiceUnavailable().body("Service Unavailable")
        );
    }
    Ok(HttpResponse::Ok().json(state.get_health().await))
}
