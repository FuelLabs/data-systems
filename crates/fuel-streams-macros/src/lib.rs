#![doc = include_str!("../README.md")]

pub mod subject {
    use std::collections::HashMap;
    pub use std::fmt::Debug;

    use downcast_rs::{impl_downcast, Downcast};
    use serde::{Deserialize, Serialize};
    pub use serde_json;
    pub use subject_derive::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct FieldSchema {
        #[serde(rename = "type")]
        pub type_name: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Schema {
        pub name: String,
        pub subject: String,
        pub format: String,
        pub wildcard: String,
        pub fields: HashMap<String, FieldSchema>,
        pub variants: Option<HashMap<String, Schema>>,
    }
    impl Schema {
        pub fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
        pub fn set_variant(
            &mut self,
            name: String,
            variant: Schema,
        ) -> &mut Self {
            if self.variants.is_none() {
                self.variants = Some(HashMap::new());
            }
            self.variants.as_mut().unwrap().insert(name, variant);
            self
        }
    }

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
        fn schema(&self) -> Schema;
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
