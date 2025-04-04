use fuel_streams_types::BlockHeight;
use serde::{self, Deserialize, Deserializer, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum DeliverPolicyError {
    #[error("Invalid deliver policy format. Expected 'new' or 'from_block=<height>'")]
    InvalidFormat,
    #[error("Block height cannot be empty")]
    EmptyBlockHeight,
    #[error("Invalid block height '{0}': must be a positive number")]
    InvalidBlockHeight(String),
}

#[derive(Hash, Debug, Default, Serialize, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DeliverPolicy {
    #[default]
    New,
    FromBlock {
        #[serde(rename = "block_height")]
        block_height: BlockHeight,
    },
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
            value
                if value.starts_with("from_block:")
                    || value.starts_with("from_block=") =>
            {
                let block_height = value
                    .strip_prefix("from_block:")
                    .or_else(|| value.strip_prefix("from_block="))
                    .ok_or(DeliverPolicyError::InvalidFormat)?
                    .trim();

                if block_height.is_empty() {
                    return Err(DeliverPolicyError::EmptyBlockHeight);
                }

                let block_height =
                    block_height.parse::<BlockHeight>().map_err(|_| {
                        DeliverPolicyError::InvalidBlockHeight(
                            block_height.to_string(),
                        )
                    })?;

                Ok(DeliverPolicy::FromBlock { block_height })
            }
            _ => Err(DeliverPolicyError::InvalidFormat),
        }
    }
}

// Add custom deserialization
impl<'de> Deserialize<'de> for DeliverPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PolicyHelper {
            String(String),
            Object {
                #[serde(rename = "from_block")]
                from_block: BlockHeightU64,
            },
            ObjectString {
                #[serde(rename = "from_block")]
                from_block: BlockHeightStr,
            },
        }

        #[derive(Deserialize)]
        struct BlockHeightU64 {
            #[serde(rename = "block_height")]
            block_height: u64,
        }

        #[derive(Deserialize)]
        struct BlockHeightStr {
            #[serde(rename = "block_height")]
            block_height: String,
        }

        let helper = PolicyHelper::deserialize(deserializer)?;
        match helper {
            PolicyHelper::String(s) => {
                s.parse().map_err(serde::de::Error::custom)
            }
            PolicyHelper::Object { from_block } => {
                Ok(DeliverPolicy::FromBlock {
                    block_height: from_block.block_height.into(),
                })
            }
            PolicyHelper::ObjectString { from_block } => {
                Ok(DeliverPolicy::FromBlock {
                    block_height: from_block
                        .block_height
                        .parse::<BlockHeight>()
                        .map_err(serde::de::Error::custom)?,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_string_deserialization() {
        // Test "new" string format
        let json = r#""new""#;
        let policy: DeliverPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy, DeliverPolicy::New);

        // Test "from_block:123" string format
        let json = r#""from_block:123""#;
        let policy: DeliverPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy, DeliverPolicy::FromBlock {
            block_height: 123.into()
        });

        // Test "from_block=123" string format
        let json = r#""from_block=123""#;
        let policy: DeliverPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy, DeliverPolicy::FromBlock {
            block_height: 123.into()
        });
    }

    #[test]
    fn test_object_deserialization() {
        // Test object format
        let json = r#"{"from_block": {"block_height": 123}}"#;
        let policy: DeliverPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy, DeliverPolicy::FromBlock {
            block_height: 123.into()
        });
    }

    #[test]
    fn test_invalid_formats() {
        // Test invalid string format
        let json = r#""invalid_format""#;
        let result: Result<DeliverPolicy, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Test invalid block height
        let json = r#""from_block:invalid""#;
        let result: Result<DeliverPolicy, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Test empty block height
        let json = r#""from_block:""#;
        let result: Result<DeliverPolicy, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization() {
        // Test New variant serialization
        let policy = DeliverPolicy::New;
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#""new""#);

        // Test FromBlock variant serialization
        let policy = DeliverPolicy::FromBlock {
            block_height: 123.into(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        assert_eq!(json, r#"{"from_block":{"block_height":"123"}}"#);
    }
}
