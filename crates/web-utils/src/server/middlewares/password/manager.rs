use std::collections::HashMap;

use actix_web::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};

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
        (headers, query_map): (HeaderMap, HashMap<String, String>),
    ) -> Result<String, PasswordVerificationError> {
        // Add password from query params to headers if present
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
        let token = match token.to_str() {
            Ok(token) => token,
            Err(_) => return Err(PasswordVerificationError::Invalid),
        };

        if !token.starts_with(BEARER) {
            return Err(PasswordVerificationError::Invalid);
        }
        urlencoding::decode(token.trim_start_matches(BEARER))
            .map_err(|_| PasswordVerificationError::Invalid)
            .map(|decoded| decoded.trim().to_string())
    }
}
