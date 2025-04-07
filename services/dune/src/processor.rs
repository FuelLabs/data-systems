use std::{fs::File, io::Write, path::Path, sync::Arc};

use fuel_streams_domains::{blocks::Block, transactions::Transaction};
use fuel_streams_types::BlockHeight;
use rayon::prelude::*;

use crate::{
    helpers::{AvroParser, BatchCalculator},
    s3::{
        FuelNetwork,
        S3KeyBuilder,
        S3Storage,
        S3StorageOpts,
        S3TableName,
        Storage,
        StorageConfig,
    },
    schemas::{AvroBlock, AvroReceipt, AvroTransaction},
    DuneResult,
};

#[derive(Debug, Clone)]
pub enum StorageType {
    S3(Arc<S3Storage>),
    File,
}

#[derive(Debug, Clone)]
pub struct BlocksProcessor {
    storage_type: StorageType,
    batch_calculator: BatchCalculator,
}

impl BlocksProcessor {
    const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

    pub async fn new(storage_type: &str) -> DuneResult<Self> {
        let s3_storage_opts = S3StorageOpts::admin_opts();
        let s3_storage = Arc::new(S3Storage::new(s3_storage_opts).await?);
        s3_storage.ensure_bucket().await?;
        let storage_type = match storage_type {
            "S3" => StorageType::S3(s3_storage),
            "File" => StorageType::File,
            _ => panic!("Invalid storage type specified"),
        };
        Ok(Self {
            storage_type,
            batch_calculator: BatchCalculator::new(Self::MAX_FILE_SIZE),
        })
    }

    async fn create_output(
        &self,
        data: Vec<u8>,
        key: &str,
    ) -> DuneResult<String> {
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

    pub async fn process_blocks_range(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
        first_height: BlockHeight,
        last_height: BlockHeight,
    ) -> DuneResult<String> {
        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroBlock>()
            .expect("Failed to create Avro writer");

        for (block, transactions) in blocks {
            let avro_transactions = transactions
                .par_iter()
                .map(|tx| AvroTransaction::from((block, tx)))
                .collect();
            let avro_block = AvroBlock::new(block, avro_transactions);
            avro_writer.append(&avro_block)?;
        }

        let combined_data = avro_writer.into_inner()?;
        let network = FuelNetwork::load_from_env();
        let key_builder =
            S3KeyBuilder::new(network).with_table(S3TableName::Blocks);
        let key = key_builder.build_key(first_height, last_height);
        let file_path = self.create_output(combined_data, &key).await?;
        tracing::info!("New file saved: {}", file_path);
        Ok(file_path)
    }

    pub async fn process_txs_range(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
        first_height: BlockHeight,
        last_height: BlockHeight,
    ) -> DuneResult<String> {
        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroTransaction>()
            .expect("Failed to create Avro writer");

        for (block, transactions) in blocks {
            for tx in transactions {
                let avro_tx = AvroTransaction::new(
                    tx,
                    Some(block.height.into()),
                    Some(block.header.get_timestamp_utc().timestamp()),
                    Some(block.id.as_ref().to_vec()),
                    Some(block.version.to_string()),
                    Some(block.producer.as_ref().to_vec()),
                );
                avro_writer.append(&avro_tx)?;
            }
        }

        let combined_data = avro_writer.into_inner()?;
        let network = FuelNetwork::load_from_env();
        let key_builder =
            S3KeyBuilder::new(network).with_table(S3TableName::Transactions);
        let key = key_builder.build_key(first_height, last_height);
        let created = self.create_output(combined_data, &key).await?;
        tracing::info!("New file saved: {}", created);
        Ok(created)
    }

    pub async fn process_receipts_range(
        &self,
        blocks: &[(Block, Vec<Transaction>)],
        first_height: BlockHeight,
        last_height: BlockHeight,
    ) -> DuneResult<String> {
        let mut avro_writer = AvroParser::default()
            .writer_with_schema::<AvroReceipt>()
            .expect("Failed to create Avro writer");

        for (block, transactions) in blocks {
            for tx in transactions {
                for receipt in &tx.receipts {
                    let avro_receipt = AvroReceipt::from((block, tx, receipt));
                    avro_writer.append(&avro_receipt)?;
                }
            }
        }

        let combined_data = avro_writer.into_inner()?;
        let network = FuelNetwork::load_from_env();
        let key_builder =
            S3KeyBuilder::new(network).with_table(S3TableName::Receipts);
        let key = key_builder.build_key(first_height, last_height);
        let created = self.create_output(combined_data, &key).await?;
        tracing::info!("New file saved: {}", created);
        Ok(created)
    }

    pub fn batch_calculator(&self) -> BatchCalculator {
        self.batch_calculator.to_owned()
    }
}
