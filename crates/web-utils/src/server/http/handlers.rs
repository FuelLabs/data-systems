use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};

use crate::server::state::StateProvider;

pub async fn get_metrics<T: StateProvider>(
    State(state): State<T>,
) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(axum::body::Body::from(state.get_metrics().await))
        .unwrap_or_else(|e| {
            // Fallback in case of response building error (unlikely)
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build response: {}", e),
            )
                .into_response()
        })
}

pub async fn get_health<T: StateProvider>(
    State(state): State<T>,
) -> impl IntoResponse {
    if !state.is_healthy().await {
        return (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable")
            .into_response();
    }
    let response = state.get_health().await;
    Json(response).into_response()
}
