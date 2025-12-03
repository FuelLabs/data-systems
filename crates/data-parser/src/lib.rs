use std::fmt::Debug;

/// Serialization/Deserialization error types.
#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Data parser error types.
#[derive(Debug, thiserror::Error)]
pub enum DataParserError {
    #[error("An error occurred during JSON encoding: {0}")]
    EncodeJson(#[source] SerdeError),
    #[error("An error occurred during JSON decoding: {0}")]
    DecodeJson(#[source] SerdeError),
}

#[derive(Debug, Clone, strum::EnumIter, strum_macros::Display)]
pub enum SerializationType {
    #[strum(serialize = "json")]
    Json,
}

#[async_trait::async_trait]
pub trait DataEncoder:
    serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + Send
    + Sync
    + Debug
    + std::marker::Sized
{
    fn data_parser() -> DataParser {
        DataParser::default()
    }

    fn encode_json(&self) -> Result<Vec<u8>, DataParserError> {
        Self::data_parser().encode_json(self)
    }

    fn decode_json(encoded: &[u8]) -> Result<Self, DataParserError> {
        Self::data_parser().decode_json(encoded)
    }

    fn to_json_value(&self) -> Result<serde_json::Value, DataParserError> {
        Self::data_parser().to_json_value(self)
    }
}

#[derive(Clone)]
pub struct DataParser {
    pub serialization_type: SerializationType,
}

impl Default for DataParser {
    fn default() -> Self {
        Self {
            serialization_type: SerializationType::Json,
        }
    }
}

impl DataParser {
    pub fn encode_json<T: DataEncoder>(
        &self,
        data: &T,
    ) -> Result<Vec<u8>, DataParserError> {
        serde_json::to_vec(&data)
            .map_err(|e| DataParserError::EncodeJson(SerdeError::Json(e)))
    }

    pub fn to_json_value<T: serde::Serialize>(
        &self,
        data: &T,
    ) -> Result<serde_json::Value, DataParserError> {
        serde_json::to_value(data)
            .map_err(|e| DataParserError::EncodeJson(SerdeError::Json(e)))
    }

    pub fn decode_json<T: DataEncoder>(&self, data: &[u8]) -> Result<T, DataParserError> {
        serde_json::from_slice(data)
            .map_err(|e| DataParserError::DecodeJson(SerdeError::Json(e)))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    struct TestData {
        field: String,
    }

    impl DataEncoder for TestData {}

    #[tokio::test]
    async fn test_encode_decode() {
        let parser = DataParser::default();
        let original_data = TestData {
            field: "test".to_string(),
        };
        let encoded = parser.encode_json(&original_data).unwrap();
        let decoded: TestData = parser.decode_json(&encoded).unwrap();
        assert_eq!(original_data, decoded);
    }
}
