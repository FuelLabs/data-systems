pub mod subject {
    pub use subject_derive::*;

    /// A subject is a topic/channel/queue/event that is used to publish and subscribe to messages.
    pub trait IntoSubject: std::fmt::Debug + Clone + Default {
        const WILDCARD: &'static str;

        fn parse_param<V: ToString>(param: &Option<V>) -> String {
            match param {
                Some(val) => val.to_string(),
                None => "*".to_string(),
            }
        }

        fn parse(&self) -> String;
        fn new() -> Self;
    }
}
