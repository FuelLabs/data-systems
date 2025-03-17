use std::collections::HashMap;

use axum::http::{
    header::{HeaderMap, AUTHORIZATION},
    HeaderValue,
};
use urlencoding::decode;

use super::PasswordVerificationError;

const BEARER: &str = "Bearer";

#[derive(Debug, Clone)]
pub struct PasswordManager {
    pub password: String,
}

impl PasswordManager {
    pub fn new(password: String) -> Self {
        Self { password }
    }

    pub fn validate_password(
        &self,
        password: &str,
    ) -> Result<(), PasswordVerificationError> {
        if password.is_empty() {
            return Err(PasswordVerificationError::Empty);
        }
        if password.eq_ignore_ascii_case(&self.password) {
            Ok(())
        } else {
            Err(PasswordVerificationError::Invalid)
        }
    }

    pub fn password_from_headers(
        &self,
        headers: HeaderMap,
        query_map: HashMap<String, String>,
    ) -> Result<String, PasswordVerificationError> {
        let mut headers = headers;
        for (key, value) in query_map.iter() {
            if key.eq_ignore_ascii_case("password") {
                let token = format!("Bearer {}", value);
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&token)
                        .map_err(PasswordVerificationError::InvalidHeader)?,
                );
            }
        }

        match Self::from_query_string(&headers) {
            Ok(key) => Ok(key.to_string()),
            Err(_) => Err(PasswordVerificationError::NotFound),
        }
    }

    fn from_query_string(
        headers: &HeaderMap,
    ) -> Result<String, PasswordVerificationError> {
        let token = headers
            .get(AUTHORIZATION)
            .ok_or(PasswordVerificationError::NotFound)?;
        let token = token
            .to_str()
            .map_err(|_| PasswordVerificationError::Invalid)?;

        if !token.starts_with(BEARER) {
            return Err(PasswordVerificationError::Invalid);
        }
        decode(token.trim_start_matches(BEARER))
            .map_err(|_| PasswordVerificationError::Invalid)
            .map(|decoded| decoded.trim().to_string())
    }
}
