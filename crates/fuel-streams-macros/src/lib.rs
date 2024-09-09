#![doc = include_str!("../README.md")]

pub mod subject {
    pub use subject_derive::*;

    /// This trait is used internally by the `Subject` derive macro to convert a struct into a
    /// standard NATS subject.
    pub trait IntoSubject: std::fmt::Debug + Send + Sync {
        fn parse(&self) -> String;
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
