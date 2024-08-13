mod utils;

use clap::Parser;
use fuel_core_services::Service;
use utils::{blocks::BlockHelper, nats::NatsHelper, tx::TxHelper};

#[derive(Parser)]
pub struct Cli {
    #[command(flatten)]
    fuel_core_config: fuel_core_bin::cli::run::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();

    let cli = Cli::parse();
    let service = fuel_core_bin::cli::run::get_service(cli.fuel_core_config)?;
    let chain_config = service.shared.config.snapshot_reader.chain_config();
    let chain_id = chain_config.consensus_parameters.chain_id();
    let block_importer = service.shared.block_importer.block_importer.clone();
    let database = service.shared.database.clone();
    service.start()?;

    // ------------------------------------------------------------------------
    // NATS
    // ------------------------------------------------------------------------
    let nats = NatsHelper::connect(true).await?;
    let block_helper = BlockHelper::new(nats.to_owned(), &database);
    let tx_helper = TxHelper::new(nats.to_owned(), &chain_id, &database);

    // ------------------------------------------------------------------------
    // OLD BLOCKS
    // ------------------------------------------------------------------------
    tokio::task::spawn({
        let block_helper = block_helper.clone();
        let _tx_helper = tx_helper.clone();
        let last_height = database.on_chain().latest_height()?.unwrap();
        async move {
            for height in 0..*last_height {
                let height = height.into();
                let block = block_helper.find_by_height(height);
                block_helper.publish(&block).await?;
                // for (index, tx) in block.transactions().iter().enumerate() {
                //     tx_helper.publish(&block, tx, index).await?;
                // }
            }
            Ok::<(), async_nats::Error>(())
        }
    });

    // ------------------------------------------------------------------------
    // NEW BLOCKS
    // ------------------------------------------------------------------------
    let mut subscription = block_importer.subscribe();
    while let Ok(result) = subscription.recv().await {
        let result = &**result;
        let block = &result.sealed_block.entity;
        block_helper.publish(block).await?;
        // for (index, tx) in block.transactions().iter().enumerate() {
        //     tx_helper.publish(block, tx, index).await?;
        // }
    }

    Ok(())
}
