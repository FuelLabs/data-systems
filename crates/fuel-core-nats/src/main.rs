use clap::Parser;

use fuel_core_bin::cli::run;
use fuel_core_services::Service;

#[derive(Parser)]
pub struct Cli {
    #[arg(
        long,
        value_name = "URL",
        env = "NATS_URL",
        default_value = "localhost:4222"
    )]
    nats_url: String,
    #[command(flatten)]
    fuel_core_config: run::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();

    let cli = Cli::parse();
    let service = run::get_service(cli.fuel_core_config)?;
    let chain_config = service.shared.config.snapshot_reader.chain_config();
    let chain_id = chain_config.consensus_parameters.chain_id();
    let base_asset_id = chain_config.consensus_parameters.base_asset_id();
    service.start()?;

    let subscription = service.shared.block_importer.block_importer.subscribe();

    let publisher = fuel_core_nats::Publisher::new(
        &cli.nats_url,
        chain_id,
        *base_asset_id,
        service.shared.database.clone(),
        subscription,
    )
    .await?;
    publisher.run().await?;

    Ok(())
}
