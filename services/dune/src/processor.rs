use crate::{
    DuneError,
    DuneResult,
    helpers::{
        AvroParser,
        AvroWriter,
    },
    s3::{
        FuelNetwork,
        S3KeyBuilder,
        S3Storage,
        S3StorageOpts,
        S3TableName,
        Storage,
        StorageConfig,
    },
    schemas::{
        AvroBlock,
        AvroReceipt,
        AvroTransaction,
    },
};
use apache_avro::{
    AvroSchema,
    schema::derive::AvroSchemaComponent,
};
use fuel_streams_domains::{
    blocks::Block,
    transactions::Transaction,
};
use fuel_streams_types::BlockHeight;
use std::{
    fmt::Display,
    fs::File,
    io::Write,
    ops::Deref,
    path::Path,
    sync::Arc,
};

#[derive(Debug, Clone)]
pub enum SizeUnit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
}

#[derive(Debug, Clone)]
pub enum StorageType {
    S3(Arc<S3Storage>),
    File,
}

#[derive(Debug, Clone)]
pub struct Processor {
    storage_type: StorageType,
    pub max_file_size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum StorageTypeConfig {
    S3,
    File,
}

impl Display for StorageTypeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageTypeConfig::S3 => write!(f, "S3"),
            StorageTypeConfig::File => write!(f, "File"),
        }
    }
}

impl std::str::FromStr for StorageTypeConfig {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<StorageTypeConfig, Self::Err> {
        match input {
            "S3" => Ok(StorageTypeConfig::S3),
            "File" => Ok(StorageTypeConfig::File),
            _ => Err(anyhow::anyhow!("Unknown storage type {input}")),
        }
    }
}

const LATEST_BLOCK_HEIGHT_KEY: &str = "latest_block_height.txt";

impl Processor {
    const DEFAULT_MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

    pub async fn new(storage_type: StorageTypeConfig) -> DuneResult<Self> {
        let storage_type = match storage_type {
            StorageTypeConfig::S3 => {
                let s3_storage_opts = S3StorageOpts::admin_opts();
                let s3_storage = Arc::new(S3Storage::new(s3_storage_opts).await?);
                s3_storage.ensure_bucket().await?;
                StorageType::S3(s3_storage)
            }
            StorageTypeConfig::File => StorageType::File,
        };
        Ok(Self {
            storage_type,
            max_file_size: Self::get_size(
                Self::DEFAULT_MAX_FILE_SIZE,
                SizeUnit::Megabytes,
            ),
        })
    }

    pub async fn new_with_unit(
        storage_type: StorageTypeConfig,
        size: usize,
        unit: SizeUnit,
    ) -> DuneResult<Self> {
        let mut processor = Self::new(storage_type).await?;
        processor.max_file_size = Self::get_size(size, unit);
        Ok(processor)
    }

    fn get_size(size: usize, unit: SizeUnit) -> usize {
        match unit {
            SizeUnit::Bytes => size,
            SizeUnit::Kilobytes => size * 1024,
            SizeUnit::Megabytes => size * 1024 * 1024,
            SizeUnit::Gigabytes => size * 1024 * 1024 * 1024,
        }
    }

    async fn create_output(&self, data: Vec<u8>, key: &str) -> DuneResult<String> {
        let created = match &self.storage_type {
            StorageType::File => {
                let manifest_dir = env!("CARGO_MANIFEST_DIR");
                let flat_key = key.replace('/', "_").replace("-", "_");
                let output_dir = Path::new(manifest_dir).join("output");
                std::fs::create_dir_all(&output_dir)?;
                let file_path = output_dir.join(&flat_key).to_path_buf();
                tracing::info!("Writing file: {:?}", file_path);
                File::create(&file_path)?.write_all(&data)?;
                let file_path = file_path.as_os_str();
                format!("{}", file_path.to_string_lossy())
            }
            StorageType::S3(s3_storage) => {
                s3_storage.store(key, data).await?;
                key.to_string()
            }
        };
        Ok(created)
    }

    pub async fn save_latest_height(
        &self,
        height: fuel_core_types::fuel_types::BlockHeight,
    ) -> DuneResult<String> {
        let network = FuelNetwork::load_from_env();
        let key_builder = S3KeyBuilder::new(network).with_table(S3TableName::Metadata);
        let key = key_builder.build_key(LATEST_BLOCK_HEIGHT_KEY);
        let data = height.deref().to_string().into_bytes();
        let file_path = self.create_output(data, &key).await?;
        Ok(file_path)
    }

    pub async fn load_latest_height(
        &self,
    ) -> DuneResult<Option<fuel_core_types::fuel_types::BlockHeight>> {
        let s3_storage_opts = S3StorageOpts::admin_opts();
        let s3_storage = S3Storage::new(s3_storage_opts).await?;

        // Retrieve the data from S3
        let network = FuelNetwork::load_from_env();
        let key_builder = S3KeyBuilder::new(network).with_table(S3TableName::Metadata);
        let key = key_builder.build_key(LATEST_BLOCK_HEIGHT_KEY);
        let data = s3_storage.retrieve(&key).await?;

        let Some(data) = data else {
            return Ok(None);
        };

        let number = String::from_utf8(data).map_err(|e| {
            anyhow::anyhow!("Unable to convert bytes into a string: {}", e)
        })?;
        let height: u32 = number
            .parse()
            .map_err(|e| anyhow::anyhow!("Unable to parse string into a u32: {}", e))?;

        Ok(Some(height.into()))
    }

    pub async fn process_range(
        &self,
        batches: Vec<(BlockHeight, BlockHeight, Vec<u8>)>,
        table: S3TableName,
    ) -> DuneResult<Vec<String>> {
        let mut file_paths = Vec::new();
        for (start, end, data) in batches {
            let network = FuelNetwork::load_from_env();
            let key_builder = S3KeyBuilder::new(network).with_table(table);
            let key = key_builder.build_key_from_heights(start, end);
            let file_path = self.create_output(data, &key).await?;
            tracing::info!("New file saved: {}", file_path);
            file_paths.push(file_path);
        }

        Ok(file_paths)
    }

    /// Process pre-serialized data directly (from disk buffer)
    /// This method takes data that has already been serialized to Avro format
    pub async fn process_data(
        &self,
        start_height: BlockHeight,
        end_height: BlockHeight,
        data: Vec<u8>,
        table: S3TableName,
    ) -> DuneResult<String> {
        let network = FuelNetwork::load_from_env();
        let key_builder = S3KeyBuilder::new(network).with_table(table);
        let key = key_builder.build_key_from_heights(start_height, end_height);
        let file_path = self.create_output(data, &key).await?;
        tracing::info!("New file saved: {}", file_path);
        Ok(file_path)
    }

    fn spli_batches<
        T: serde::Serialize
            + serde::de::DeserializeOwned
            + AvroSchema
            + AvroSchemaComponent
            + Send
            + Sync
            + 'static,
    >(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
        avro_writer: AvroWriter<T>,
    ) -> DuneResult<Vec<(BlockHeight, BlockHeight, Vec<u8>)>> {
        if blocks.is_empty() {
            return Ok(vec![]);
        }

        let serialized = avro_writer.into_inner()?;
        let total_size = serialized.len();
        let first_height = blocks.first().unwrap().0.height;
        let last_height = blocks.last().unwrap().0.height;
        if total_size <= self.max_file_size {
            return Ok(vec![(first_height, last_height, serialized)]);
        }

        let mut batches = Vec::new();
        let mid = blocks.len() / 2;
        let (left, right) = blocks.split_at(mid);
        if !left.is_empty() {
            batches.extend(self.calculate_blocks_batches(left)?);
        }
        if !right.is_empty() {
            batches.extend(self.calculate_blocks_batches(right)?);
        }

        Ok(batches)
    }

    pub fn calculate_blocks_batches(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
    ) -> DuneResult<Vec<(BlockHeight, BlockHeight, Vec<u8>)>> {
        let mut items = Vec::with_capacity(blocks.len());
        for (block, _) in blocks {
            let avro_block = AvroBlock::new(block);
            items.push(avro_block);
        }

        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroBlock>()
            .expect("Failed to create Avro writer");

        for item in items {
            avro_writer.append(&item)?;
        }

        self.spli_batches(blocks, avro_writer)
    }

    pub fn calculate_txs_batches(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
    ) -> DuneResult<Vec<(BlockHeight, BlockHeight, Vec<u8>)>> {
        let mut items = Vec::with_capacity(blocks.len());
        for (block, transactions) in blocks {
            for tx in transactions {
                let avro_tx = AvroTransaction::new(
                    tx,
                    Some(block.height.into()),
                    Some(block.header.get_timestamp_utc().timestamp()),
                    Some(block.id.as_ref().to_vec().into()),
                    Some(block.version.to_string()),
                    Some(block.producer.as_ref().to_vec().into()),
                );
                items.push(avro_tx);
            }
        }

        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroTransaction>()
            .expect("Failed to create Avro writer");

        for item in items {
            avro_writer.append(&item)?;
        }

        self.spli_batches(blocks, avro_writer)
    }

    pub fn calculate_receipts_batches(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
    ) -> DuneResult<Vec<(BlockHeight, BlockHeight, Vec<u8>)>> {
        let mut items = Vec::with_capacity(blocks.len());
        for (block, transactions) in blocks {
            for tx in transactions {
                for receipt in &tx.receipts {
                    let avro_receipt = AvroReceipt::from((block, tx, receipt));
                    items.push(avro_receipt);
                }
            }
        }

        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroReceipt>()
            .expect("Failed to create Avro writer");

        for item in items {
            avro_writer.append(&item)?;
        }

        self.spli_batches(blocks, avro_writer)
    }

    pub fn calculate_height_batches(
        start_height: BlockHeight,
        last_height: BlockHeight,
        batch_size: usize,
    ) -> DuneResult<Vec<(BlockHeight, BlockHeight)>> {
        if *last_height < *start_height {
            return Err(DuneError::InvalidBlockRange {
                start: *start_height,
                end: *last_height,
            });
        }

        let mut batches = Vec::new();
        let total_blocks = *last_height - *start_height + 1;
        let num_batches = ((total_blocks as f64) / (batch_size as f64)).ceil() as u32;
        let size_per_batch = total_blocks.div_ceil(num_batches);
        let mut current = *start_height;
        while current <= *last_height {
            let batch_end = std::cmp::min(current + size_per_batch - 1, *last_height);
            batches.push((current.into(), batch_end.into()));
            current = batch_end + 1;
        }

        Ok(batches)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use fuel_streams_domains::mocks::{
        MockBlock,
        MockReceipt,
        MockTransaction,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    fn deserialize_avro<T>(data: &[u8]) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned
            + AvroSchemaComponent
            + AvroSchema
            + Send
            + Sync
            + 'static,
    {
        let parser = AvroParser::default();
        let reader = parser.reader_with_schema::<T>()?;
        Ok(reader.deserialize(data)?)
    }

    #[test]
    fn test_calculate_height_batches_perfect_division() -> Result<()> {
        let batches = Processor::calculate_height_batches(1.into(), 3000.into(), 1000)?;

        assert_eq!(
            batches,
            vec![
                (1.into(), 1000.into()),
                (1001.into(), 2000.into()),
                (2001.into(), 3000.into())
            ],
            "Should create equal sized batches when perfectly divisible"
        );
        Ok(())
    }

    #[test]
    fn test_calculate_height_batches_uneven_division() -> Result<()> {
        let batches =
            Processor::calculate_height_batches(1000.into(), 3500.into(), 1000)?;

        assert_eq!(
            batches.len(),
            3,
            "Should create appropriate number of batches"
        );
        assert!(
            batches.windows(2).all(|w| *w[1].0 == *w[0].1 + 1),
            "Batches should be continuous with no gaps"
        );
        assert_eq!(
            *batches[0].0, 1000,
            "First batch should start at the given start value"
        );
        assert_eq!(
            *batches.last().unwrap().1,
            3500,
            "Last batch should end at the given last height"
        );
        Ok(())
    }

    #[test]
    fn test_calculate_height_batches_invalid_range() -> Result<()> {
        let result = Processor::calculate_height_batches(2.into(), 1.into(), 1000);
        assert!(result.is_err(), "Should return error for invalid range");
        match result {
            Err(DuneError::InvalidBlockRange { start, end }) => {
                assert_eq!(start, 2);
                assert_eq!(end, 1);
            }
            _ => panic!("Expected InvalidBlockRange error"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_blocks_batches() -> Result<()> {
        let processor =
            Processor::new_with_unit(StorageTypeConfig::File, 1, SizeUnit::Megabytes)
                .await?;

        let mut blocks_and_txs = Vec::new();
        for i in 1..=3 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let tx = MockTransaction::script(vec![], vec![], vec![]);
            blocks_and_txs.push((block, vec![tx]));
        }

        let batches = processor.calculate_blocks_batches(&blocks_and_txs)?;
        assert_eq!(batches.len(), 1, "Should fit in one batch with 1MB limit");
        assert_eq!(
            batches[0].0,
            BlockHeight::from(1),
            "First batch should start at height 1"
        );
        assert_eq!(
            batches.last().unwrap().1,
            BlockHeight::from(3),
            "Last batch should end at height 3"
        );

        let mut total_blocks = 0;
        for (start, end, data) in &batches {
            let deserialized = deserialize_avro::<AvroBlock>(data)?;
            total_blocks += deserialized.len();
            for block in deserialized {
                assert!(
                    block.height.unwrap() >= **start as i64
                        && block.height.unwrap() <= **end as i64,
                    "Block height should be within batch range"
                );
            }
        }
        assert_eq!(
            total_blocks, 3,
            "Total deserialized blocks should match input"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_txs_batches() -> Result<()> {
        let processor =
            Processor::new_with_unit(StorageTypeConfig::File, 1, SizeUnit::Megabytes)
                .await?;

        let mut blocks_and_txs = Vec::new();
        for i in 1..=3 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            // Create multiple transactions per block to better test transaction batching
            let txs = vec![
                MockTransaction::script(vec![], vec![], vec![]),
                MockTransaction::script(vec![], vec![], vec![]),
            ];
            blocks_and_txs.push((block, txs));
        }

        let batches = processor.calculate_txs_batches(&blocks_and_txs)?;
        assert_eq!(batches.len(), 1, "Should fit in one batch with 1MB limit");
        assert_eq!(
            batches[0].0,
            BlockHeight::from(1),
            "First batch should start at height 1"
        );
        assert_eq!(
            batches.last().unwrap().1,
            BlockHeight::from(3),
            "Last batch should end at height 3"
        );

        let mut total_txs = 0;
        for (start, end, data) in &batches {
            let deserialized = deserialize_avro::<AvroTransaction>(data)?;
            total_txs += deserialized.len();
            for tx in deserialized {
                assert!(
                    tx.block_height.unwrap() >= **start as i64
                        && tx.block_height.unwrap() <= **end as i64,
                    "Transaction block height should be within batch range"
                );
            }
        }
        assert_eq!(
            total_txs, 6,
            "Total deserialized transactions should match input (2 txs * 3 blocks)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_receipts_batches() -> Result<()> {
        let processor =
            Processor::new_with_unit(StorageTypeConfig::File, 1, SizeUnit::Megabytes)
                .await?;

        let mut blocks_and_txs = Vec::new();
        for i in 1..=3 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            // Create transactions with multiple receipts
            let tx = MockTransaction::script(
                vec![],
                vec![],
                MockReceipt::all(), // This creates multiple receipts
            );
            blocks_and_txs.push((block, vec![tx]));
        }

        let batches = processor.calculate_receipts_batches(&blocks_and_txs)?;
        assert_eq!(batches.len(), 1, "Should fit in one batch with 1MB limit");
        assert_eq!(
            batches[0].0,
            BlockHeight::from(1),
            "First batch should start at height 1"
        );
        assert_eq!(
            batches.last().unwrap().1,
            BlockHeight::from(3),
            "Last batch should end at height 3"
        );

        let mut total_receipts = 0;
        for (start, end, data) in &batches {
            let deserialized = deserialize_avro::<AvroReceipt>(data)?;
            total_receipts += deserialized.len();
            for receipt in deserialized {
                assert!(
                    receipt.block_height.unwrap() >= **start as i64
                        && receipt.block_height.unwrap() <= **end as i64,
                    "Receipt block height should be within batch range"
                );
            }
        }
        assert!(
            total_receipts > 0,
            "Should have at least some receipts in the test data"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_load_save_block_height_s3() -> Result<()> {
        let processor =
            Processor::new_with_unit(StorageTypeConfig::S3, 1, SizeUnit::Megabytes)
                .await?;

        // Given
        let expected_height = 123u32.into();
        processor.save_latest_height(expected_height).await?;

        // When
        let result = processor.load_latest_height().await?;

        // Then
        let actual_height = result.expect("Height should be present");
        assert_eq!(actual_height, expected_height, "Heights should match");

        Ok(())
    }
}
