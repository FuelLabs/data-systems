use actix_web::http::header::InvalidHeaderValue;

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

impl From<PasswordVerificationError> for actix_web::Error {
    fn from(err: PasswordVerificationError) -> Self {
        match err {
            PasswordVerificationError::NotFound => {
                actix_web::error::ErrorUnauthorized("Password not found")
            }
            PasswordVerificationError::Empty => {
                actix_web::error::ErrorUnauthorized("Password is empty")
            }
            PasswordVerificationError::Invalid => {
                actix_web::error::ErrorUnauthorized("Password is invalid")
            }
            PasswordVerificationError::InvalidHeader(e) => {
                actix_web::error::ErrorUnauthorized(e.to_string())
            }
        }
    }
}
