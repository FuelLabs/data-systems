use fuel_core_types::fuel_types::BlockHeight;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum SubjectName {
    Blocks,
    Receipts1,
    Receipts2,
    Receipts3,
    Transactions,
    Owners,
    Assets,
}

impl SubjectName {
    // TODO: Extract this in a ConnectionAware trait? where there
    // are more connection-related operations
    pub fn get_string(&self, connection_id: &str) -> String {
        [connection_id, &self.to_string()].concat()
    }
}

impl std::fmt::Display for SubjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = match self {
            SubjectName::Blocks => "blocks.*",
            SubjectName::Receipts1 => "receipts.*.*.*",
            SubjectName::Receipts2 => "receipts.*.*.*.*",
            SubjectName::Receipts3 => "receipts.*.*.*.*.*",
            SubjectName::Transactions => "transactions.*.*.*",
            SubjectName::Owners => "owners.*.*",
            SubjectName::Assets => "assets.*.*",
        };

        write!(f, "{value}")
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Subject {
    Blocks {
        height: BlockHeight,
    },
    Transactions {
        height: BlockHeight,
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
    pub fn get_string(&self, connection_id: &str) -> String {
        [connection_id, &self.to_string()].concat()
    }

    #[allow(dead_code)]
    fn get_name(&self) -> SubjectName {
        match self {
            Subject::Blocks { .. } => SubjectName::Blocks,
            Subject::Transactions { .. } => SubjectName::Transactions,
        }
    }
}

pub fn get_all_in_connection(connection_id: &str) -> Vec<String> {
    get_all()
        .iter()
        .map(|name| format!("{connection_id}{name}"))
        .collect()
}
fn get_all() -> Vec<String> {
    SubjectName::iter().map(|name| name.to_string()).collect()
}
