use std::{collections::HashMap, sync::LazyLock};

use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;

use super::models::{LoginRequest, LoginResponse};
use crate::server::{
    auth::{create_jwt, AuthError, UserError, UserRole},
    state::ServerState,
};

pub static AUTH_DATA: LazyLock<HashMap<String, (String, UserRole, Uuid)>> =
    LazyLock::new(|| {
        HashMap::from_iter(vec![
            (
                "client".to_string(),
                ("client".to_string(), UserRole::Client, Uuid::new_v4()),
            ),
            (
                "admin".to_string(),
                ("admin".to_string(), UserRole::Admin, Uuid::new_v4()),
            ),
        ])
    });

pub async fn get_metrics(
    state: web::Data<ServerState>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type(
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )
        .body(state.context.telemetry.get_metrics().await))
}

pub async fn get_health(state: web::Data<ServerState>) -> Result<HttpResponse> {
    if !state.is_healthy() {
        return Ok(
            HttpResponse::ServiceUnavailable().body("Service Unavailable")
        );
    }
    Ok(HttpResponse::Ok().json(state.get_health().await))
}

// request jwt
pub async fn request_jwt(
    state: web::Data<ServerState>,
    req_body: web::Json<LoginRequest>,
) -> actix_web::Result<HttpResponse> {
    // get user by username
    let (password, user_role, uuid) = (*AUTH_DATA)
        .get(&req_body.username)
        .ok_or(UserError::UserNotFound)?;

    // compare pwd with expected one
    if !password.eq_ignore_ascii_case(&req_body.password) {
        return Err(AuthError::WrongCredentialsError.into());
    }

    // if all good, generate a jwt with the user role encoded
    let jwt_token = create_jwt(
        &uuid.to_string(),
        user_role,
        state.context.jwt_secret.as_bytes(),
    )
    .map_err(|_| AuthError::JWTTokenCreationError)?;

    Ok(HttpResponse::Ok().json(&LoginResponse {
        id: uuid.to_owned(),
        username: req_body.username.clone(),
        jwt_token,
    }))
}
