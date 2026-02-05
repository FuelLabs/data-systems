use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use fuel_streams_types::BlockHeight;

use crate::{
    DuneError,
    DuneResult,
    helpers::{
        AvroFileWriter,
        AvroParser,
    },
    schemas::{
        AvroBlock,
        AvroReceipt,
        AvroTransaction,
        ReceiptMetadata,
    },
};
use fuel_streams_domains::{
    blocks::Block,
    transactions::Transaction,
};

/// The result of finalizing a batch to files, containing paths to Avro files.
/// Files are streamed directly to S3 without loading into memory.
pub struct FinalizedBatchFiles {
    pub first_height: BlockHeight,
    pub last_height: BlockHeight,
    /// Path to the blocks Avro file
    pub blocks_path: PathBuf,
    /// Path to the transactions Avro file  
    pub transactions_path: PathBuf,
    /// Path to the receipts Avro file
    pub receipts_path: PathBuf,
    /// Temporary directory containing the files (for cleanup)
    temp_dir: PathBuf,
}

impl FinalizedBatchFiles {
    /// Cleans up all temporary files after upload
    pub fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

impl Drop for FinalizedBatchFiles {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// ============================================================================
// Avro file writers for disk-based buffering
// ============================================================================

/// Paths to finalized Avro files ready for upload.
/// Note: Does NOT implement Drop - ownership of temp_dir is transferred to FinalizedBatchFiles.
struct FinalizedAvroFiles {
    temp_dir: PathBuf,
    blocks_path: PathBuf,
    transactions_path: PathBuf,
    receipts_path: PathBuf,
}

/// Manages Avro file writers for blocks, transactions, and receipts.
/// Writes directly to disk to avoid memory accumulation.
///
/// Implements Drop to clean up temp directory on error. On success,
/// ownership is transferred via `finalize_to_paths()` and Drop becomes a no-op.
struct AvroFileWriters {
    /// Temp directory path. Set to None after successful finalize_to_paths()
    /// to transfer ownership and prevent cleanup on drop.
    temp_dir: Option<PathBuf>,
    blocks_writer: Option<AvroFileWriter<AvroBlock>>,
    transactions_writer: Option<AvroFileWriter<AvroTransaction>>,
    receipts_writer: Option<AvroFileWriter<AvroReceipt>>,
}

impl Drop for AvroFileWriters {
    fn drop(&mut self) {
        // Clean up temp directory if we still own it (i.e., finalize_to_paths wasn't called)
        if let Some(ref temp_dir) = self.temp_dir {
            let _ = fs::remove_dir_all(temp_dir);
        }
    }
}

impl AvroFileWriters {
    /// Creates new Avro file writers in a temporary directory
    fn new() -> DuneResult<Self> {
        let temp_dir = std::env::temp_dir().join(format!(
            "dune-avro-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        Self::with_dir(&temp_dir)
    }

    /// Creates new Avro file writers in the specified directory.
    /// Cleans up the directory if any writer creation fails.
    fn with_dir(dir: impl AsRef<Path>) -> DuneResult<Self> {
        let temp_dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&temp_dir)?;

        // Use inner function to enable cleanup on any failure after dir creation
        let result = Self::create_writers(&temp_dir);

        if result.is_err() {
            // Clean up the directory we created
            let _ = fs::remove_dir_all(&temp_dir);
        }

        result
    }

    /// Helper to create all writers. Called by with_dir().
    fn create_writers(temp_dir: &Path) -> DuneResult<Self> {
        let parser = AvroParser::default();

        let blocks_writer = parser
            .file_writer_with_schema(temp_dir.join("blocks.avro"))
            .map_err(|e| {
                DuneError::Other(anyhow::anyhow!("Failed to create blocks writer: {}", e))
            })?;
        let transactions_writer = parser
            .file_writer_with_schema(temp_dir.join("transactions.avro"))
            .map_err(|e| {
                DuneError::Other(anyhow::anyhow!(
                    "Failed to create transactions writer: {}",
                    e
                ))
            })?;
        let receipts_writer = parser
            .file_writer_with_schema(temp_dir.join("receipts.avro"))
            .map_err(|e| {
                DuneError::Other(anyhow::anyhow!(
                    "Failed to create receipts writer: {}",
                    e
                ))
            })?;

        Ok(Self {
            temp_dir: Some(temp_dir.to_path_buf()),
            blocks_writer: Some(blocks_writer),
            transactions_writer: Some(transactions_writer),
            receipts_writer: Some(receipts_writer),
        })
    }

    /// Appends block data directly to the Avro writers
    fn append(&mut self, block: &Block, transactions: &[Transaction]) -> DuneResult<()> {
        let blocks_writer = self.blocks_writer.as_mut().ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("blocks_writer not available"))
        })?;
        let transactions_writer = self.transactions_writer.as_mut().ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("transactions_writer not available"))
        })?;
        let receipts_writer = self.receipts_writer.as_mut().ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("receipts_writer not available"))
        })?;

        // Convert and write block
        let avro_block = AvroBlock::new(block);
        blocks_writer.append(&avro_block)?;

        // Convert and write transactions
        for tx in transactions {
            let avro_tx = AvroTransaction::new(
                tx,
                Some(block.height.into()),
                Some(block.header.get_timestamp_utc().timestamp()),
                Some(block.id.as_ref().to_vec().into()),
                Some(block.version.to_string()),
                Some(block.producer.as_ref().to_vec().into()),
            );
            transactions_writer.append(&avro_tx)?;
        }

        // Convert and write receipts
        for tx in transactions {
            let receipt_metadata = ReceiptMetadata {
                block_time: Some(block.header.get_timestamp_utc().timestamp()),
                block_height: Some(block.height.0 as i64),
                block_version: Some(block.version.to_string()),
                block_producer: Some(block.producer.clone().into()),
                transaction_id: Some(tx.id.clone().into()),
            };
            for receipt in &tx.receipts {
                let avro_receipt = AvroReceipt::new(receipt, &receipt_metadata);
                receipts_writer.append(&avro_receipt)?;
            }
        }

        Ok(())
    }

    /// Finalizes all writers and returns paths to the Avro files.
    /// Does NOT load files into memory - use this for large batches.
    ///
    /// On success, ownership of temp_dir is transferred to FinalizedAvroFiles,
    /// and this struct's Drop will not clean up the directory.
    /// On error, Drop will clean up the temp directory.
    fn finalize_to_paths(&mut self) -> DuneResult<FinalizedAvroFiles> {
        let blocks_writer = self.blocks_writer.take().ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("blocks_writer already taken"))
        })?;
        let transactions_writer = self.transactions_writer.take().ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("transactions_writer already taken"))
        })?;
        let receipts_writer = self.receipts_writer.take().ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("receipts_writer already taken"))
        })?;

        let blocks_path = blocks_writer.finalize_path()?;
        let transactions_path = transactions_writer.finalize_path()?;
        let receipts_path = receipts_writer.finalize_path()?;

        // Take ownership of temp_dir so Drop won't clean it up
        let temp_dir = self
            .temp_dir
            .take()
            .ok_or_else(|| DuneError::Other(anyhow::anyhow!("temp_dir already taken")))?;

        Ok(FinalizedAvroFiles {
            temp_dir,
            blocks_path,
            transactions_path,
            receipts_path,
        })
    }
}

// ============================================================================
// Disk-based buffer implementation
// ============================================================================

/// Disk-based block buffer that writes blocks directly to Avro files.
/// Uses minimal memory by streaming data directly to disk.
pub struct DiskBuffer {
    writers: Option<AvroFileWriters>,
    first_height: Option<BlockHeight>,
    last_height: Option<BlockHeight>,
    block_count: usize,
}

impl DiskBuffer {
    pub fn new() -> DuneResult<Self> {
        let writers = AvroFileWriters::new()?;
        Ok(Self {
            writers: Some(writers),
            first_height: None,
            last_height: None,
            block_count: 0,
        })
    }

    #[cfg(test)]
    pub fn with_dir(dir: impl AsRef<Path>) -> DuneResult<Self> {
        let writers = AvroFileWriters::with_dir(dir)?;
        Ok(Self {
            writers: Some(writers),
            first_height: None,
            last_height: None,
            block_count: 0,
        })
    }

    /// Returns the number of blocks in the buffer
    pub fn len(&self) -> usize {
        self.block_count
    }

    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.block_count == 0
    }

    /// Returns the first block height in the buffer
    pub fn first_height(&self) -> Option<BlockHeight> {
        self.first_height
    }

    /// Returns the last block height in the buffer
    pub fn last_height(&self) -> Option<BlockHeight> {
        self.last_height
    }

    /// Appends a block and its transactions to the buffer.
    /// Data is written directly to Avro files on disk.
    pub fn append(
        &mut self,
        block: &Block,
        transactions: &[Transaction],
    ) -> DuneResult<()> {
        let height = block.height;

        if self.first_height.is_none() {
            self.first_height = Some(height);
        }
        self.last_height = Some(height);

        let writers = self
            .writers
            .as_mut()
            .ok_or_else(|| DuneError::Other(anyhow::anyhow!("Writers not available")))?;

        writers.append(block, transactions)?;
        self.block_count += 1;

        Ok(())
    }

    /// Finalizes the buffer, returning paths to the Avro files for upload.
    ///
    /// This does NOT clear the buffer - call `reset()` after successful upload
    /// to clear the data. This design allows retry on upload failure without
    /// losing the buffered data.
    pub fn finalize(&mut self) -> DuneResult<FinalizedBatchFiles> {
        let first_height = self.first_height.ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("Cannot finalize empty buffer"))
        })?;
        let last_height = self.last_height.ok_or_else(|| {
            DuneError::Other(anyhow::anyhow!("Cannot finalize empty buffer"))
        })?;

        let writers = self
            .writers
            .as_mut()
            .ok_or_else(|| DuneError::Other(anyhow::anyhow!("Writers not available")))?;

        let avro_files = writers.finalize_to_paths()?;

        Ok(FinalizedBatchFiles {
            first_height,
            last_height,
            blocks_path: avro_files.blocks_path,
            transactions_path: avro_files.transactions_path,
            receipts_path: avro_files.receipts_path,
            temp_dir: avro_files.temp_dir,
        })
    }

    /// Resets the buffer for reuse, clearing all data.
    /// Call this after successful upload to prepare for the next batch.
    pub fn reset(&mut self) -> DuneResult<()> {
        // Drop old writers (this cleans up the temp directory)
        let _ = self.writers.take();

        // Create new writers
        self.writers = Some(AvroFileWriters::new()?);

        self.first_height = None;
        self.last_height = None;
        self.block_count = 0;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_streams_domains::mocks::{
        MockBlock,
        MockReceipt,
        MockTransaction,
    };
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn test_disk_buffer_basic_operations() -> DuneResult<()> {
        let dir = tempdir().unwrap();
        let mut buffer = DiskBuffer::with_dir(dir.path())?;

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        // Add some blocks
        for i in 1..=10 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
            buffer.append(&block, &txs)?;
        }

        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.first_height(), Some(BlockHeight::from(1)));
        assert_eq!(buffer.last_height(), Some(BlockHeight::from(10)));

        // Finalize and verify
        let finalized = buffer.finalize()?;
        assert_eq!(*finalized.first_height, 1);
        assert_eq!(*finalized.last_height, 10);

        // Verify files exist
        assert!(finalized.blocks_path.exists());
        assert!(finalized.transactions_path.exists());
        assert!(finalized.receipts_path.exists());

        Ok(())
    }

    #[test]
    fn test_disk_buffer_reset() -> DuneResult<()> {
        let dir = tempdir().unwrap();
        let mut buffer = DiskBuffer::with_dir(dir.path())?;

        // Add blocks
        for i in 1..=5 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
            buffer.append(&block, &txs)?;
        }

        assert_eq!(buffer.len(), 5);

        // Reset
        buffer.reset()?;

        assert!(buffer.is_empty());
        assert_eq!(buffer.first_height(), None);
        assert_eq!(buffer.last_height(), None);

        // Can add more blocks after reset
        let mut block = MockBlock::random();
        block.height = BlockHeight::from(100);
        let txs = vec![MockTransaction::script(vec![], vec![], vec![])];
        buffer.append(&block, &txs)?;

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.first_height(), Some(BlockHeight::from(100)));

        Ok(())
    }

    #[test]
    fn test_disk_buffer_with_receipts() -> DuneResult<()> {
        let dir = tempdir().unwrap();
        let mut buffer = DiskBuffer::with_dir(dir.path())?;

        for i in 1..=3 {
            let mut block = MockBlock::random();
            block.height = BlockHeight::from(i);
            let txs = vec![MockTransaction::script(vec![], vec![], MockReceipt::all())];
            buffer.append(&block, &txs)?;
        }

        let finalized = buffer.finalize()?;

        // Verify receipts file exists and has content
        assert!(finalized.receipts_path.exists());
        let receipts_size = std::fs::metadata(&finalized.receipts_path)?.len();
        assert!(receipts_size > 0, "Receipts file should not be empty");

        Ok(())
    }

    #[test]
    fn test_disk_buffer_new() -> DuneResult<()> {
        let buffer = DiskBuffer::new()?;
        assert!(buffer.is_empty());
        Ok(())
    }
}
