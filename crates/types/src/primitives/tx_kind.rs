use std::fmt;

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize,
    Deserializer,
    Serialize,
};

use crate::fuel_core::FuelCoreTransaction;

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub enum TransactionKind {
    #[default]
    Create,
    Mint,
    Script,
    Upgrade,
    Upload,
    Blob,
}

impl TransactionKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Mint => "mint",
            Self::Script => "script",
            Self::Upgrade => "upgrade",
            Self::Upload => "upload",
            Self::Blob => "blob",
        }
    }

    fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "create" => Some(Self::Create),
            "mint" => Some(Self::Mint),
            "script" => Some(Self::Script),
            "upgrade" => Some(Self::Upgrade),
            "upload" => Some(Self::Upload),
            "blob" => Some(Self::Blob),
            _ => None,
        }
    }
}

impl std::fmt::Display for TransactionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TransactionKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s == Self::Create.as_str() => Ok(Self::Create),
            s if s == Self::Mint.as_str() => Ok(Self::Mint),
            s if s == Self::Script.as_str() => Ok(Self::Script),
            s if s == Self::Upgrade.as_str() => Ok(Self::Upgrade),
            s if s == Self::Upload.as_str() => Ok(Self::Upload),
            s if s == Self::Blob.as_str() => Ok(Self::Blob),
            _ => Err(format!("Invalid transaction kind: {s}")),
        }
    }
}

impl<'de> Deserialize<'de> for TransactionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionKindVisitor;

        impl<'de> Visitor<'de> for TransactionKindVisitor {
            type Value = TransactionKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("a string or a map representing TransactionKind")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Use from_str_case_insensitive for concise matching
                TransactionKind::from_str_case_insensitive(value).ok_or_else(
                    || {
                        E::custom(format!(
                            "invalid TransactionKind string: {}",
                            value
                        ))
                    },
                )
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut kind_type: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => {
                            if kind_type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            kind_type = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let kind_str = kind_type
                    .ok_or_else(|| de::Error::missing_field("type"))?;
                self.visit_str(&kind_str)
            }
        }

        deserializer.deserialize_any(TransactionKindVisitor)
    }
}

impl From<&FuelCoreTransaction> for TransactionKind {
    fn from(value: &FuelCoreTransaction) -> Self {
        match value {
            FuelCoreTransaction::Script(_) => TransactionKind::Script,
            FuelCoreTransaction::Create(_) => TransactionKind::Create,
            FuelCoreTransaction::Mint(_) => TransactionKind::Mint,
            FuelCoreTransaction::Upgrade(_) => TransactionKind::Upgrade,
            FuelCoreTransaction::Upload(_) => TransactionKind::Upload,
            FuelCoreTransaction::Blob(_) => TransactionKind::Blob,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[derive(Deserialize, Debug)]
    struct MyStruct {
        kind: TransactionKind,
    }

    #[test]
    fn test_deserialize_string_lowercase() {
        let json = json!("mint");
        let kind: TransactionKind = serde_json::from_value(json).unwrap();
        assert_eq!(kind, TransactionKind::Mint);
    }

    #[test]
    fn test_deserialize_string_uppercase() {
        let json = json!("Mint");
        let kind: TransactionKind = serde_json::from_value(json).unwrap();
        assert_eq!(kind, TransactionKind::Mint);
    }

    #[test]
    fn test_deserialize_map() {
        let json = json!({ "type": "Mint" });
        let kind: TransactionKind = serde_json::from_value(json).unwrap();
        assert_eq!(kind, TransactionKind::Mint);
    }

    #[test]
    fn test_deserialize_map_lowercase() {
        let json = json!({ "type": "mint" });
        let kind: TransactionKind = serde_json::from_value(json).unwrap();
        assert_eq!(kind, TransactionKind::Mint);
    }

    #[test]
    fn test_deserialize_invalid() {
        let json = json!("invalid");
        let result: Result<TransactionKind, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json2 = json!({ "type": "invalid" });
        let result2: Result<TransactionKind, _> = serde_json::from_value(json2);
        assert!(result2.is_err());
    }
    #[test]
    fn test_deserialize_missing_type() {
        let json = json!({});
        let result: Result<TransactionKind, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize() {
        let kind = TransactionKind::Mint;
        let serialized = serde_json::to_value(kind).unwrap();
        assert_eq!(serialized, json!("Mint"));
    }
    #[test]
    fn test_deserialize_whole_json_lowercase() {
        let json = json!({ "kind": "mint" });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.kind, TransactionKind::Mint);
    }

    #[test]
    fn test_deserialize_whole_json_uppercase() {
        let json = json!({ "kind": "Mint" });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.kind, TransactionKind::Mint);
    }
    #[test]
    fn test_deserialize_whole_json_map() {
        let json = json!({ "kind": { "type": "Mint" } });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.kind, TransactionKind::Mint);
    }

    #[test]
    fn test_deserialize_whole_json_map_lowercase() {
        let json = json!({ "kind": { "type": "mint" } });
        let deserialized: MyStruct = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.kind, TransactionKind::Mint);
    }
}
