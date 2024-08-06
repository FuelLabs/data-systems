use std::fmt;

static DEFAULT_NAMESPACE: &str = "fuel";

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

impl From<&str> for NatsNamespace {
    fn from(value: &str) -> Self {
        NatsNamespace::Custom(value.into())
    }
}

impl From<NatsNamespace> for String {
    fn from(val: NatsNamespace) -> Self {
        val.to_string()
    }
}

impl NatsNamespace {
    pub fn subject_name(&self, val: &str) -> String {
        format!("{self}.{}", val)
    }

    pub fn stream_name(&self, val: &str) -> String {
        format!("{self}:stream:{val}")
    }

    pub fn consumer_name(&self, val: &str) -> String {
        format!("{self}:consumer:{val}")
    }

    pub fn prepend_subjects(&self, subjects: Vec<&str>) -> Vec<String> {
        subjects.iter().map(|s| format!("{self}.{s}")).collect()
    }
}
