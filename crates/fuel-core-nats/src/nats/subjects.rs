use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum SubjectName {
    Blocks,
    Transactions,
}

impl SubjectName {
    // TODO: Extract this in a ConnectionAware trait? where there
    // are more connection-related operations
    pub fn get_string(&self, connection_id: impl AsRef<str>) -> String {
        [connection_id.as_ref(), &self.to_string()].concat()
    }
}

impl std::fmt::Display for SubjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            SubjectName::Blocks => "blocks.*",
            SubjectName::Transactions => "transactions.*.*.*",
        };

        write!(f, "{value}")
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Subject {
    Blocks {
        height: u32,
    },
    Transactions {
        height: u32,
        index: usize,
        kind: String,
    },
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Blocks { height } => write!(f, "blocks.{height}"),
            Subject::Transactions {
                height,
                index,
                kind,
            } => write!(f, "transactions.{height}.{index}.{kind}"),
        }
    }
}

impl Subject {
    pub fn get_string(&self, connection_id: impl AsRef<str>) -> String {
        [connection_id.as_ref(), &self.to_string()].concat()
    }

    #[allow(dead_code)]
    fn get_name(&self) -> SubjectName {
        match self {
            Subject::Blocks { .. } => SubjectName::Blocks,
            Subject::Transactions { .. } => SubjectName::Transactions,
        }
    }
}

pub fn get_all_in_connection(connection_id: impl AsRef<str>) -> Vec<String> {
    let connection_id = connection_id.as_ref();
    get_all()
        .iter()
        .map(|name| format!("{connection_id}{name}"))
        .collect()
}
fn get_all() -> Vec<String> {
    SubjectName::iter().map(|name| name.to_string()).collect()
}
