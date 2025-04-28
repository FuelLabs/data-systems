use anyhow::{anyhow, Result};
use fuel_streams_domains::blocks::Block;
use fuel_streams_types::BlockHeight;
use redis::{Commands, RedisResult};

pub struct Store {
    client: redis::Client,
}

impl Store {
    pub fn new(_storage_dir: Option<&str>) -> Result<Self> {
        let redis_url = dotenvy::var("REDIS_URL")
            .map_err(|_| anyhow!("REDIS_URL must be set"))?;
        let client = redis::Client::open(redis_url.as_str())
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
        Ok(Self { client })
    }

    pub async fn get_last_block_saved(&self) -> Result<BlockHeight> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;

        let height: RedisResult<Option<String>> = conn.get("last_block_saved");
        let height =
            height.map_err(|e| anyhow!("Failed to fetch last block: {}", e))?;

        let block_height = match height {
            Some(h) => h
                .parse::<u64>()
                .map(BlockHeight::from)
                .map_err(|e| anyhow!("Invalid height format: {}", e))?,
            None => BlockHeight::from(0),
        };

        Ok(block_height)
    }

    pub async fn save_last_block(&self, block: &Block) -> Result<()> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;
        let result: RedisResult<()> =
            conn.set("last_block_saved", block.height.0.to_string());
        result.map_err(|e| anyhow!("Failed to save last block: {}", e))?;

        Ok(())
    }

    pub async fn get_total_blocks(&self) -> Result<usize> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;
        let total: RedisResult<Option<String>> = conn.get("blocks_saved");
        let total = total
            .map_err(|e| anyhow!("Failed to fetch total blocks: {}", e))?;

        let total = match total {
            Some(t) => t
                .parse::<usize>()
                .map_err(|e| anyhow!("Invalid total format: {}", e))?,
            None => 0,
        };

        Ok(total)
    }

    pub async fn save_total_blocks(&self, blocks_num: usize) -> Result<()> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))?;
        let _new_total: RedisResult<usize> =
            conn.incr("blocks_saved", blocks_num);
        _new_total
            .map_err(|e| anyhow!("Failed to increment total blocks: {}", e))?;

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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use fuel_streams_domains::{blocks::Block, mocks::MockBlock};
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    use super::*;

    fn create_test_block(height: u64) -> Block {
        MockBlock::build(height.into())
    }

    async fn cleanup_redis(store: &Store) -> Result<()> {
        let mut conn = store.client.get_connection()?;
        let _: () = redis::cmd("FLUSHALL").query(&mut conn)?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_store_creation() -> Result<()> {
        let store = Store::new(None)?;
        assert!(store.client.get_connection().is_ok());
        cleanup_redis(&store).await?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_last_block_operations() -> Result<()> {
        let store = Store::new(None)?;

        // Test initial state
        let initial_height = store.get_last_block_saved().await?;
        assert_eq!(*initial_height, 0);

        // Test saving and retrieving a block
        let test_block = create_test_block(42);
        store.save_last_block(&test_block).await?;

        let saved_height = store.get_last_block_saved().await?;
        assert_eq!(*saved_height, 42);

        cleanup_redis(&store).await?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_total_blocks_operations() -> Result<()> {
        let store = Store::new(None)?;

        // Test initial state
        let initial_total = store.get_total_blocks().await?;
        assert_eq!(initial_total, 0);

        // Test incrementing total blocks
        store.save_total_blocks(5).await?;
        let total = store.get_total_blocks().await?;
        assert_eq!(total, 5);

        // Test additional increment
        store.save_total_blocks(3).await?;
        let new_total = store.get_total_blocks().await?;
        assert_eq!(new_total, 8);

        cleanup_redis(&store).await?;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_should_continue_processing() -> Result<()> {
        let store = Store::new(None)?;

        // Test with no max blocks (should always return true)
        assert!(store.should_continue_processing(None).await?);

        // Test with max blocks
        store.save_total_blocks(5).await?;
        assert!(store.should_continue_processing(Some(10)).await?);
        assert!(!store.should_continue_processing(Some(5)).await?);
        assert!(!store.should_continue_processing(Some(3)).await?);

        cleanup_redis(&store).await?;
        Ok(())
    }
}
