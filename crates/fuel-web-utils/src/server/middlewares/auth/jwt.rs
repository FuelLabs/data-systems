use std::{collections::HashMap, convert::TryFrom, fmt};

use actix_web::{
    http::header::{HeaderMap, HeaderValue, AUTHORIZATION},
    HttpResponse,
    ResponseError,
};
use chrono::Utc;
use jsonwebtoken::{
    decode,
    encode,
    Algorithm,
    DecodingKey,
    EncodingKey,
    Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const BEARER: &str = "Bearer";

#[derive(
    Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd,
)]
pub enum UserType {
    ADMIN,
    CLIENT,
}

/// User-related errors
#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
    #[error("Unknown User Role: {0}")]
    UnknownUserRole(String),
    #[error("Unknown User Status: {0}")]
    UnknownUserStatus(String),
    #[error("Unallowed User Role: {0}")]
    UnallowedUserRole(String),
    #[error("Missing password")]
    MissingPassword,
    #[error("Missing username")]
    MissingUsername,
    #[error("Wrong password")]
    WrongPassword,
    #[error("User is not verified")]
    UnverifiedUser,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        match self {
            UserError::UserNotFound => {
                HttpResponse::NotFound().body(self.to_string())
            }
            UserError::UnknownUserRole(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            UserError::UnknownUserStatus(_) => {
                HttpResponse::NotFound().body(self.to_string())
            }
            UserError::UnallowedUserRole(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            UserError::MissingPassword => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            UserError::MissingUsername => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            UserError::WrongPassword => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            UserError::UnverifiedUser => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
        }
    }
}

/// Auth errors
#[derive(Clone, Debug, thiserror::Error, PartialEq)]
pub enum AuthError {
    #[error("Wrong Credentials")]
    WrongCredentialsError,
    #[error("JWT Token not valid")]
    JWTTokenError,
    #[error("JWT Token Creation Error")]
    JWTTokenCreationError,
    #[error("No Auth Header")]
    NoAuthHeaderError,
    #[error("Invalid Auth Header")]
    InvalidAuthHeaderError,
    #[error("No Permission")]
    NoPermissionError,
    #[error("Expired Token")]
    ExpiredToken,
    #[error("Bad Encoded User Role: {0}")]
    BadEncodedUserRole(String),
    #[error("Unparsable UUID error: {0}")]
    UnparsableUuid(String),
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::WrongCredentialsError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::JWTTokenError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::JWTTokenCreationError => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            AuthError::NoAuthHeaderError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::InvalidAuthHeaderError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::NoPermissionError => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::ExpiredToken => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::BadEncodedUserRole(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AuthError::UnparsableUuid(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

/// A user role
#[repr(i16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin = 0,
    Client = 1,
}

impl From<UserRole> for i16 {
    fn from(role: UserRole) -> i16 {
        role as i16
    }
}

impl TryFrom<i16> for UserRole {
    type Error = UserError;

    fn try_from(n: i16) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(UserRole::Admin),
            1 => Ok(UserRole::Client),
            _ => Err(UserError::UnknownUserRole(n.to_string())),
        }
    }
}

/// Maps a string to a Role
impl TryFrom<&str> for UserRole {
    type Error = UserError;

    fn try_from(role: &str) -> Result<Self, Self::Error> {
        match role.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "client" => Ok(UserRole::Client),
            _ => Err(UserError::UnknownUserRole(role.to_string())),
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Client => write!(f, "client"),
        }
    }
}

impl From<UserType> for UserRole {
    fn from(value: UserType) -> Self {
        match value {
            UserType::ADMIN => UserRole::Admin,
            UserType::CLIENT => UserRole::Client,
        }
    }
}

impl From<UserRole> for UserType {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Admin => UserType::ADMIN,
            UserRole::Client => UserType::CLIENT,
        }
    }
}

/// A user status
#[repr(i16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Unverified = 0,
    Verified = 1,
}

impl From<UserStatus> for i16 {
    fn from(user_status: UserStatus) -> i16 {
        user_status as i16
    }
}

impl TryFrom<i16> for UserStatus {
    type Error = UserError;

    fn try_from(n: i16) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(UserStatus::Unverified),
            1 => Ok(UserStatus::Verified),
            _ => Err(UserError::UnknownUserStatus(n.to_string())),
        }
    }
}

/// Maps a string to a status
impl TryFrom<&str> for UserStatus {
    type Error = UserError;

    fn try_from(user_status: &str) -> Result<Self, Self::Error> {
        match user_status.to_lowercase().as_str() {
            "unverified" => Ok(UserStatus::Unverified),
            "verified" => Ok(UserStatus::Verified),
            _ => Err(UserError::UnknownUserStatus(user_status.to_string())),
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserStatus::Unverified => write!(f, "unverified"),
            UserStatus::Verified => write!(f, "verified"),
        }
    }
}

pub fn create_jwt(
    uid: &str,
    role: &UserRole,
    jwt_secret: &[u8],
) -> Result<String, AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_owned(),
        role: role.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(jwt_secret))
        .map_err(|_| AuthError::JWTTokenCreationError)
}

fn jwt_from_query_string(headers: &HeaderMap) -> Result<String, AuthError> {
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

pub fn authorize_request(
    (jwt_secret, mut headers, query_map): (
        String,
        actix_web::http::header::HeaderMap,
        HashMap<String, String>,
    ),
) -> Result<(Uuid, String), AuthError> {
    for (key, value) in query_map.iter() {
        if key.eq_ignore_ascii_case("token") {
            let token = format!("Bearer {}", value);
            headers
                .insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
        }
    }

    match jwt_from_query_string(&headers) {
        Ok(jwt) => {
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &Validation::new(Algorithm::HS512),
            )
            .map_err(|_| AuthError::JWTTokenError)?;

            // check user id
            let user_id =
                Uuid::parse_str(&decoded.claims.sub).map_err(|_| {
                    AuthError::UnparsableUuid(decoded.claims.sub.to_string())
                })?;

            // check token expiration
            let now = Utc::now().timestamp();
            if (decoded.claims.exp as i64).lt(&now) {
                return Err(AuthError::ExpiredToken);
            }

            // TODO: check for user in the db by user_id
            // get the user's role
            let _token_role = UserRole::try_from(decoded.claims.role.as_str())
                .map_err(|_| {
                    AuthError::BadEncodedUserRole(decoded.claims.role)
                })?;

            // TODO: verify db user's role vs token_role
            Ok((user_id, jwt))
        }
        Err(e) => Err(e),
    }
}
