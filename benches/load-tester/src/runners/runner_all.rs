use anyhow::Result;
use fuel_streams::prelude::*;
use fuel_streams_core::nats::FuelNetwork;
use tokio::try_join;

use super::runner_load_tester::run_blocks_consumer;

#[allow(dead_code)]
pub async fn run_all_benchmarks() -> Result<()> {
    let client = Client::connect(FuelNetwork::Testnet).await?;

    let _ = try_join!(run_blocks_consumer(&client));

    Ok(())
}
