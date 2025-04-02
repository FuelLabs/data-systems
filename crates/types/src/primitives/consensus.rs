use std::str::FromStr;

use serde::Serialize;

use crate::impl_enum_string_serialization;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    derive_more::Display,
    derive_more::IsVariant,
    utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ConsensusType {
    #[default]
    #[display("genesis")]
    Genesis,
    #[display("poa_consensus")]
    PoaConsensus,
}

impl TryFrom<&str> for ConsensusType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "genesis" => Ok(ConsensusType::Genesis),
            "poa_consensus" => Ok(ConsensusType::PoaConsensus),
            _ => Err(format!("Unknown ConsensusType: {}", s)),
        }
    }
}

impl_enum_string_serialization!(ConsensusType, "consensus_type");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("genesis");
        let value: ConsensusType = serde_json::from_value(json).unwrap();
        assert_eq!(value, ConsensusType::Genesis);
    }

    #[test]
    fn test_deserialize_mixed_case() {
        let json = json!("Genesis");
        let result: Result<ConsensusType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConsensusType::Genesis);

        let json = json!("GENESIS");
        let result: Result<ConsensusType, _> = serde_json::from_value(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ConsensusType::Genesis);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<ConsensusType, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let value = ConsensusType::Genesis;
        let serialized = serde_json::to_value(value).unwrap();
        assert_eq!(serialized, json!("genesis"));
    }

    #[test]
    fn test_case_insensitive_from_str() {
        assert_eq!(
            ConsensusType::from_str("GENESIS").unwrap(),
            ConsensusType::Genesis
        );
        assert_eq!(
            ConsensusType::from_str("genesis").unwrap(),
            ConsensusType::Genesis
        );
        assert_eq!(
            ConsensusType::from_str("Genesis").unwrap(),
            ConsensusType::Genesis
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(ConsensusType::Genesis.to_string(), "genesis");
        assert_eq!(ConsensusType::PoaConsensus.to_string(), "poa_consensus");
    }
}
