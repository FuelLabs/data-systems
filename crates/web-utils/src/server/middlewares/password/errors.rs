use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hyper::header::InvalidHeaderValue;

#[derive(Debug, thiserror::Error)]
pub enum PasswordVerificationError {
    #[error("Password not found in request")]
    NotFound,
    #[error("Supplied password is empty")]
    Empty,
    #[error("Password is invalid")]
    Invalid,
    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderValue),
}

impl IntoResponse for PasswordVerificationError {
    fn into_response(self) -> Response {
        match self {
            PasswordVerificationError::NotFound => {
                (StatusCode::UNAUTHORIZED, "Password not found").into_response()
            }
            PasswordVerificationError::Empty => {
                (StatusCode::UNAUTHORIZED, "Password is empty").into_response()
            }
            PasswordVerificationError::Invalid => {
                (StatusCode::UNAUTHORIZED, "Password is invalid")
                    .into_response()
            }
            PasswordVerificationError::InvalidHeader(e) => {
                (StatusCode::UNAUTHORIZED, e.to_string()).into_response()
            }
        }
    }
}
