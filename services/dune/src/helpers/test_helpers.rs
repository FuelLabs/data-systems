use fuel_streams_types::{Address, BlockHeight, BlockId};

use super::AvroBytes;

#[derive(Debug, Clone)]
pub struct TestBlockMetadata {
    pub block_height: i64,
    pub block_time: i64,
    pub block_id: AvroBytes,
    pub block_version: String,
    pub block_producer: AvroBytes,
}

impl Default for TestBlockMetadata {
    fn default() -> Self {
        Self {
            block_height: BlockHeight::random().0 as i64,
            block_time: 1000,
            block_version: "V1".to_string(),
            block_id: BlockId::default().clone().into(),
            block_producer: Address::random().clone().into(),
        }
    }
}

pub type TestBlockMetadataOptions = (
    Option<i64>,
    Option<i64>,
    Option<AvroBytes>,
    Option<String>,
    Option<AvroBytes>,
);

impl TestBlockMetadata {
    /// Creates a new test metadata instance with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new test metadata instance with custom values
    pub fn with_values(
        block_height: i64,
        block_time: i64,
        block_id: AvroBytes,
        block_version: String,
        block_producer: AvroBytes,
    ) -> Self {
        Self {
            block_height,
            block_time,
            block_id,
            block_version,
            block_producer,
        }
    }

    /// Converts the test metadata to a tuple of Option values
    /// This is useful for backward compatibility with existing test functions
    pub fn as_options(&self) -> TestBlockMetadataOptions {
        (
            Some(self.block_height),
            Some(self.block_time),
            Some(self.block_id.clone()),
            Some(self.block_version.clone()),
            Some(self.block_producer.clone()),
        )
    }
}

pub async fn write_schema_files(schemas: &[(&str, apache_avro::Schema)]) {
    use tokio::fs;

    for (filename, schema) in schemas {
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        let dir = std::path::Path::new("schemas");
        let path = dir.join(filename);
        fs::create_dir_all(dir).await.unwrap();
        fs::write(path, schema_json.as_bytes()).await.unwrap();
    }
}
