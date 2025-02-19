use crate::declare_integer_wrapper;

#[derive(thiserror::Error, Debug)]
pub enum WordError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

declare_integer_wrapper!(Word, u64, WordError);
