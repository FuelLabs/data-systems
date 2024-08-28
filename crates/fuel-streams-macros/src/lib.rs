pub mod subject {
    pub use subject_derive::*;

    /// This trait is used internally by the `Subject` derive macro to convert a struct into a
    /// standard NATS subject.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_streams_macros::subject::IntoSubject;
    ///
    /// #[derive(Debug, Clone, Default)]
    /// struct MySubject {
    ///     field1: Option<String>,
    ///     field2: Option<u32>,
    /// }
    ///
    /// impl IntoSubject for MySubject {
    ///     const WILDCARD: &'static str = "my_subject.>";
    ///
    ///     fn parse(&self) -> String {
    ///         format!("my_subject.{}.{}",
    ///             Self::parse_param(&self.field1),
    ///             Self::parse_param(&self.field2)
    ///         )
    ///     }
    ///
    ///     fn new() -> Self {
    ///         Self::default()
    ///     }
    /// }
    ///
    /// // Test WILDCARD constant
    /// assert_eq!(MySubject::WILDCARD, "my_subject.>");
    ///
    /// // Test parse_param method
    /// assert_eq!(MySubject::parse_param(&Some("value".to_string())), "value");
    /// assert_eq!(MySubject::parse_param(&None::<String>), "*");
    ///
    /// // Test parse method
    /// let subject = MySubject {
    ///     field1: Some("hello".to_string()),
    ///     field2: Some(42),
    /// };
    /// assert_eq!(subject.parse(), "my_subject.hello.42");
    ///
    /// // Test new method
    /// let new_subject = MySubject::new();
    /// assert_eq!(new_subject.parse(), "my_subject.*.*");
    /// ```
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
