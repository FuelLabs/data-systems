use serde::{self, Deserialize, Serialize};

#[derive(
    Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub enum DeliverPolicy {
    #[default]
    New,
    FromBlock {
        #[serde(rename = "blockHeight")]
        block_height: u64,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum DeliverPolicyError {
    #[error("Invalid delivery policy format. Expected 'new' or 'from_block:<height>'")]
    InvalidFormat,
    #[error("Block height cannot be empty")]
    EmptyBlockHeight,
    #[error("Invalid block height '{0}': must be a positive number")]
    InvalidBlockHeight(String),
}

impl std::fmt::Display for DeliverPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliverPolicy::New => write!(f, "new"),
            DeliverPolicy::FromBlock { block_height } => {
                write!(f, "from_block:{}", block_height)
            }
        }
    }
}

impl std::str::FromStr for DeliverPolicy {
    type Err = DeliverPolicyError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "new" => Ok(DeliverPolicy::New),
            value if value.starts_with("from_block:") => {
                let block_height = match value.strip_prefix("from_block:") {
                    Some(height) => height.trim(),
                    None => return Err(DeliverPolicyError::InvalidFormat),
                };

                if block_height.is_empty() {
                    return Err(DeliverPolicyError::EmptyBlockHeight);
                }

                let height = block_height.parse::<u64>().map_err(|_| {
                    DeliverPolicyError::InvalidBlockHeight(
                        block_height.to_string(),
                    )
                })?;

                Ok(DeliverPolicy::FromBlock {
                    block_height: height,
                })
            }
            _ => Err(DeliverPolicyError::InvalidFormat),
        }
    }
}
