use std::path::Path;

use anyhow::Result;
use bytes::Bytes;
use fuel_data_parser::DataEncoder;
use fuel_streams_domains::blocks::Block;
use fuel_streams_types::BlockHeight;
use surrealkv::Store as SurrealStore;

pub struct Store {
    inner: SurrealStore,
}

impl Store {
    pub fn new(storage_dir: Option<&str>) -> Result<Self> {
        let mut opts = surrealkv::Options::new();
        let storage_dir = match storage_dir {
            Some(file_dir) => Path::new(file_dir).to_path_buf(),
            None => {
                let manifest_dir = env!("CARGO_MANIFEST_DIR");
                let manifest_dir =
                    Path::new(manifest_dir).join("output/storage");
                manifest_dir.to_path_buf()
            }
        };
        opts.dir = storage_dir;
        opts.disk_persistence = true;
        let store = SurrealStore::new(opts)?;
        Ok(Self { inner: store })
    }

    pub async fn get_last_block_saved(&self) -> Result<BlockHeight> {
        let block_height = {
            let mut txn = self.inner.begin()?;
            let key = Bytes::from("last_block_saved");
            match txn.get(&key)? {
                Some(value) => serde_json::from_slice(&value)?,
                None => BlockHeight::from(0),
            }
        };
        Ok(block_height)
    }

    pub async fn save_last_block(&self, block: &Block) -> Result<()> {
        let mut txn = self.inner.begin()?;
        let key = Bytes::from("last_block_saved");
        let value = block.height.encode_json()?;
        txn.set(&key, &value)?;
        txn.commit()?;
        Ok(())
    }

    pub async fn get_total_blocks(&self) -> Result<usize> {
        let total = {
            let mut txn = self.inner.begin()?;
            let key = Bytes::from("blocks_saved");
            match txn.get(&key)? {
                Some(value) => serde_json::from_slice(&value)?,
                None => 0,
            }
        };
        Ok(total)
    }

    pub async fn save_total_blocks(&self, blocks_num: usize) -> Result<()> {
        let current = self.get_total_blocks().await?;
        let mut txn = self.inner.begin()?;
        let key = Bytes::from("blocks_saved");
        let new_total = current + blocks_num;
        let value = serde_json::to_vec(&new_total)?;
        txn.set(&key, &value)?;
        txn.commit()?;
        Ok(())
    }

    pub async fn should_continue_processing(
        &self,
        max_blocks: Option<usize>,
    ) -> Result<bool> {
        if let Some(max) = max_blocks {
            let total = self.get_total_blocks().await?;
            Ok(total < max)
        } else {
            Ok(true)
        }
    }
}
