use fuel_core_types::fuel_tx::{
    Address,
    AssetId,
};
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
    pub fn to_subject_string(&self, sandbox_id: &Option<String>) -> String {
        let sandbox_id = sandbox_id.clone().unwrap_or_default();
        let to_string = self.to_string();
        format!("{sandbox_id}{to_string}")
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
        height: u32,
    },
    Receipts1 {
        height: u32,
        contract_id: String,
        topic_or_kind: String,
    },
    Receipts2 {
        height: u32,
        contract_id: String,
        topic_1: String,
        topic_2: String,
    },
    Receipts3 {
        height: u32,
        contract_id: String,
        topic_1: String,
        topic_2: String,
        topic_3: String,
    },
    Transactions {
        height: u32,
        index: usize,
        kind: String,
    },
    Owners {
        height: u32,
        owner_id: Address,
    },
    Assets {
        height: u32,
        asset_id: AssetId,
    },
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::Blocks { height } => write!(f, "blocks.{height}"),
            Subject::Receipts1 {
                height,
                contract_id,
                topic_or_kind,
            } => write!(f, "receipts.{height}.{contract_id}.{topic_or_kind}"),
            Subject::Receipts2 {
                height,
                contract_id,
                topic_1,
                topic_2,
            } => write!(f, "receipts.{height}.{contract_id}.{topic_1}.{topic_2}"),
            Subject::Receipts3 {
                height,
                contract_id,
                topic_1,
                topic_2,
                topic_3,
            } => write!(
                f,
                "receipts.{height}.{contract_id}.{topic_1}.{topic_2}.{topic_3}"
            ),
            Subject::Transactions {
                height,
                index,
                kind,
            } => write!(f, "transactions.{height}.{index}.{kind}"),
            Subject::Owners { height, owner_id } => {
                write!(f, "owners.{height}.{owner_id}")
            }
            Subject::Assets { height, asset_id } => {
                write!(f, "assets.{height}.{asset_id}")
            }
        }
    }
}

impl Subject {
    pub fn get_value(&self, sandbox_id: &Option<String>) -> String {
        let sandbox_id = sandbox_id.clone().unwrap_or_default();
        let to_string = self.to_string();
        format!("{sandbox_id}{to_string}")
    }

    #[allow(dead_code)]
    fn get_name(&self) -> SubjectName {
        match self {
            Subject::Blocks { .. } => SubjectName::Blocks,
            Subject::Receipts1 { .. } => SubjectName::Receipts1,
            Subject::Receipts2 { .. } => SubjectName::Receipts2,
            Subject::Receipts3 { .. } => SubjectName::Receipts3,
            Subject::Transactions { .. } => SubjectName::Transactions,
            Subject::Owners { .. } => SubjectName::Owners,
            Subject::Assets { .. } => SubjectName::Assets,
        }
    }
}

pub fn get_all_in_sandbox(sandbox_id: &str) -> Vec<String> {
    get_all()
        .iter()
        .map(|name| format!("{sandbox_id}{name}"))
        .collect()
}

pub fn get_all() -> Vec<String> {
    SubjectName::iter().map(|name| name.to_string()).collect()
}
