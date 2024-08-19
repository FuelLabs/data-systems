use clap::Parser;
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
    fuel_core_config: fuel_core_bin::cli::run::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();

    let cli = Cli::parse();

    let fuel_core = fuel_core_bin::cli::run::get_service(cli.fuel_core_config)?;
    fuel_core.start()?;

    let fuel_core_subscription =
        fuel_core.shared.block_importer.block_importer.subscribe();
    let fuel_core_database = fuel_core.shared.database.clone();

    let chain_config = fuel_core.shared.config.snapshot_reader.chain_config();
    let chain_id = chain_config.consensus_parameters.chain_id();
    let base_asset_id = chain_config.consensus_parameters.base_asset_id();

    let publisher = fuel_streams_publisher::Publisher::new(
        &cli.nats_url,
        chain_id,
        *base_asset_id,
        fuel_core_database,
        fuel_core_subscription,
    )
    .await?;
    publisher.run().await?;

    Ok(())
}
