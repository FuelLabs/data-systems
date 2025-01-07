pub type BoxedError = Box<dyn std::error::Error>;
pub type BoxedResult<T> = Result<T, BoxedError>;
