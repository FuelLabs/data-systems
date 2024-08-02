use std::fmt::Display;

static DEFAULT_CONN_ID: &str = "fuel-streams";

#[derive(Debug, Clone, Default)]
pub enum ConnId {
    Custom(String),
    Rnd(String),
    #[default]
    Default,
}

impl ConnId {
    pub fn new(value: impl ToString) -> Self {
        Self::Custom(value.to_string())
    }

    pub fn rnd() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_int: u32 = rng.gen();
        let value = format!(r"connection-{random_int}");
        Self::Rnd(value)
    }
}

impl Display for ConnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            ConnId::Custom(s) => s,
            ConnId::Rnd(s) => s,
            ConnId::Default => DEFAULT_CONN_ID,
        };
        write!(f, "{value}")
    }
}

impl From<&str> for ConnId {
    fn from(value: &str) -> Self {
        ConnId::Custom(value.into())
    }
}

impl From<ConnId> for String {
    fn from(val: ConnId) -> Self {
        val.to_string()
    }
}
