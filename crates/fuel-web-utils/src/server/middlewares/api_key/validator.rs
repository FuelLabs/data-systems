use std::{collections::HashMap, sync::Arc};

use actix_web::{
    http::header::{HeaderMap, HeaderValue, AUTHORIZATION},
    HttpResponse,
    ResponseError,
};

use super::{ApiKey, ApiKeyStatus, InMemoryApiKeyStorage, KeyStorage};

const BEARER: &str = "Bearer";
const X_API_KEY: &str = "X-API-Key";

/// Auth errors
#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum AuthError {
    #[error("Api Key is unknown or not registered")]
    UnknownApiKeyError,
    #[error("Api Key is not active")]
    InactiveApiKeyError,
    #[error("No Auth Header")]
    NoAuthHeaderError,
    #[error("Invalid Auth Header")]
    InvalidAuthHeaderError,
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::UnknownApiKeyError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::InactiveApiKeyError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::NoAuthHeaderError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::InvalidAuthHeaderError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
        }
    }
}

fn api_key_from_query_string(headers: &HeaderMap) -> Result<String, AuthError> {
    let token = headers
        .get(AUTHORIZATION)
        .ok_or(AuthError::NoAuthHeaderError)?;

    let token = match token.to_str() {
        Ok(token) => token,
        Err(_) => return Err(AuthError::InvalidAuthHeaderError),
    };

    if !token.starts_with(BEARER) {
        return Err(AuthError::InvalidAuthHeaderError);
    }

    urlencoding::decode(token.trim_start_matches(BEARER))
        .map_err(|_| AuthError::NoAuthHeaderError)
        .map(|decoded| decoded.trim().to_string())
}

fn api_key_from_headers(headers: &HeaderMap) -> Result<String, AuthError> {
    let x_api_key = headers
        .get(X_API_KEY)
        .cloned()
        .ok_or(AuthError::NoAuthHeaderError)?;

    if x_api_key.is_empty() {
        return Err(AuthError::InvalidAuthHeaderError);
    }

    let x_api_key = match x_api_key.to_str() {
        Ok(token) => token,
        Err(_) => return Err(AuthError::InvalidAuthHeaderError),
    };

    Ok(x_api_key.to_string())
}

fn validate_key(
    api_key_register: Arc<InMemoryApiKeyStorage>,
    api_key: &str,
) -> Result<ApiKey, AuthError> {
    let api_key_info = api_key_register
        .find_api_key(api_key)
        .map_err(|_| AuthError::UnknownApiKeyError)?;
    if api_key_info.status != ApiKeyStatus::Active {
        return Err(AuthError::InactiveApiKeyError);
    }
    Ok(api_key_info)
}

pub fn authorize_request(
    (api_key_register, mut headers, query_map): (
        Arc<InMemoryApiKeyStorage>,
        actix_web::http::header::HeaderMap,
        HashMap<String, String>,
    ),
) -> Result<ApiKey, AuthError> {
    for (key, value) in query_map.iter() {
        if key.eq_ignore_ascii_case("api_key") {
            let token = format!("Bearer {}", value);
            headers
                .insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
        }
    }

    if let Ok(api_key) = api_key_from_query_string(&headers) {
        return validate_key(api_key_register, &api_key)
    }

    if let Ok(api_key) = api_key_from_headers(&headers) {
        return validate_key(api_key_register, &api_key)
    }

    Err(AuthError::NoAuthHeaderError)
}
