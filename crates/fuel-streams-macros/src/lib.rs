pub mod subject {
    pub use subject_derive::*;
    pub trait IntoSubject:
        std::fmt::Debug + Clone + Default + Send + Sync
    {
        const WILDCARD: &'static str;

        fn all() -> &'static str {
            Self::WILDCARD
        }

        fn parse_param<V: ToString>(param: &Option<V>) -> String {
            match param {
                Some(val) => val.to_string(),
                None => "*".to_string(),
            }
        }

        fn parse(&self) -> String;
    }
}
