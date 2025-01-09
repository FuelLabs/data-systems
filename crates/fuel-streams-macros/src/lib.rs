#![doc = include_str!("../README.md")]

mod validator;
pub mod subject {
    pub use subject_derive::*;

    #[derive(thiserror::Error, Debug)]
    pub enum SubjectError {
        #[error(transparent)]
        InvalidPattern(#[from] SubjectPatternError),
        #[error(
            "Pattern is incompatible with the number of fields in the subject"
        )]
        IncompatiblePattern,
        #[error("Invalid JSON conversion: {0}")]
        InvalidJsonConversion(#[source] serde_json::Error),
        #[error("Expected JSON object")]
        ExpectedJsonObject,
    }

    pub use crate::validator::*;

    /// This trait is used internally by the `Subject` derive macro to convert a struct into a
    /// standard NATS subject.
    pub trait IntoSubject: std::fmt::Debug + Send + Sync + 'static {
        fn parse(&self) -> String;
        fn wildcard(&self) -> &'static str;
        fn to_sql_where(&self) -> Option<String>;
        fn validate_pattern(&self, pattern: &str) -> Result<(), SubjectError>;
    }

    pub trait FromJsonString:
        Clone + Sized + std::fmt::Debug + Send + Sync + 'static
    {
        fn from_json_str(json: &str) -> Result<Self, SubjectError>;
    }

    pub trait SubjectBuildable: std::fmt::Debug {
        fn new() -> Self;
    }

    pub fn parse_param<V: ToString>(param: &Option<V>) -> String {
        match param {
            Some(val) => val.to_string(),
            None => "*".to_string(),
        }
    }
}
