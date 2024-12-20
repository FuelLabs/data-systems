use std::fmt;

static DEFAULT_NAMESPACE: &str = "fuel";

/// Represents a namespace for NATS subjects and streams.
///
/// # Examples
///
/// ```
/// use fuel_streams_nats::NatsNamespace;
///
/// let default_namespace = NatsNamespace::default();
/// assert_eq!(default_namespace.to_string(), "fuel");
///
/// let custom_namespace = NatsNamespace::Custom("my_custom_namespace".to_string());
/// assert_eq!(custom_namespace.to_string(), "my_custom_namespace");
/// ```
#[derive(Debug, Clone, Default)]
pub enum NatsNamespace {
    #[default]
    Fuel,
    Custom(String),
}

impl fmt::Display for NatsNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            NatsNamespace::Fuel => DEFAULT_NAMESPACE,
            NatsNamespace::Custom(s) => s,
        };
        write!(f, "{value}")
    }
}

impl From<NatsNamespace> for String {
    fn from(val: NatsNamespace) -> Self {
        val.to_string()
    }
}

impl NatsNamespace {
    /// Creates a subject name by combining the namespace with the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_streams_nats::NatsNamespace;
    ///
    /// let namespace = NatsNamespace::default();
    /// assert_eq!(namespace.subject_name("test"), "fuel.test");
    ///
    /// let custom_namespace = NatsNamespace::Custom("custom".to_string());
    /// assert_eq!(custom_namespace.subject_name("test"), "custom.test");
    /// ```
    pub fn subject_name(&self, val: &str) -> String {
        format!("{self}.{}", val)
    }

    /// Creates a stream name by combining the namespace with the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fuel_streams_nats::NatsNamespace;
    ///
    /// let namespace = NatsNamespace::default();
    /// assert_eq!(namespace.stream_name("test"), "fuel_test");
    ///
    /// let custom_namespace = NatsNamespace::Custom("custom".to_string());
    /// assert_eq!(custom_namespace.stream_name("test"), "custom_test");
    /// ```
    pub fn stream_name(&self, val: &str) -> String {
        format!("{self}_{val}")
    }
}
