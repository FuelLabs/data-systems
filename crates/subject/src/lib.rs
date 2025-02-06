#![doc = include_str!("../README.md")]
mod payload;
mod schema;

pub mod subject {
    pub use std::fmt::Debug;

    use downcast_rs::{impl_downcast, Downcast};
    pub use indexmap::IndexMap;
    pub use serde_json;
    pub use subject_derive::*;

    #[allow(unused_imports)]
    pub use crate::{payload::*, schema::*};

    pub trait IntoSubject: Debug + Downcast + Send + Sync + 'static {
        fn id(&self) -> &'static str;
        fn parse(&self) -> String;
        fn query_all(&self) -> &'static str;
        fn to_sql_where(&self) -> Option<String>;
        fn to_sql_select(&self) -> Option<String>;
        fn schema(&self) -> Schema;
        fn to_payload(&self) -> SubjectPayload;
    }
    impl_downcast!(IntoSubject);

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
