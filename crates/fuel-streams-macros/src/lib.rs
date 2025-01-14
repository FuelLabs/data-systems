#![doc = include_str!("../README.md")]

pub mod subject {
    pub use std::fmt::Debug;

    use downcast_rs::{impl_downcast, Downcast};
    pub use serde_json;
    pub use subject_derive::*;

    #[derive(thiserror::Error, Debug, PartialEq, Eq)]
    pub enum SubjectError {
        #[error("Invalid JSON conversion: {0}")]
        InvalidJsonConversion(String),
        #[error("Expected JSON object")]
        ExpectedJsonObject,
    }

    pub trait IntoSubject: Debug + Downcast + Send + Sync + 'static {
        fn id(&self) -> &'static str;
        fn parse(&self) -> String;
        fn wildcard(&self) -> &'static str;
        fn to_sql_where(&self) -> String;
    }
    impl_downcast!(IntoSubject);

    pub trait FromJsonString:
        serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + Sized
        + Debug
        + Send
        + Sync
        + 'static
    {
        fn from_json(json: &str) -> Result<Self, SubjectError>;
        fn to_json(&self) -> String;
    }

    pub trait SubjectBuildable: Debug {
        fn new() -> Self;
    }

    pub fn parse_param<V: ToString>(param: &Option<V>) -> String {
        match param {
            Some(val) => val.to_string(),
            None => "*".to_string(),
        }
    }
}
