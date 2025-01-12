use std::{collections::HashMap, sync::LazyLock};

use actix_web::{web, HttpResponse};
use fuel_web_utils::server::middlewares::auth::jwt::{
    create_jwt,
    AuthError,
    UserError,
    UserRole,
};
use uuid::Uuid;

use super::models::{LoginRequest, LoginResponse};
use crate::server::{
    // auth::{create_jwt, AuthError, UserError, UserRole},
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

// request jwt endpoint
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
    let jwt_token =
        create_jwt(&uuid.to_string(), user_role, state.jwt_secret.as_bytes())
            .map_err(|_| AuthError::JWTTokenCreationError)?;

    Ok(HttpResponse::Ok().json(&LoginResponse {
        id: uuid.to_owned(),
        username: req_body.username.clone(),
        jwt_token,
    }))
}
