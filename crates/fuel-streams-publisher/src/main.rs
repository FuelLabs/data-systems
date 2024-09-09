//! This binary subscribes to events emitted from a Fuel client or node
//! to publish streams that can consumed via the `fuel-streams` SDK.
use clap::Parser;
use fuel_streams_core::nats::{NatsClient, NatsClientOpts};
use fuel_streams_publisher::{FuelCore, Publisher};

/// CLI structure for parsing command-line arguments.
///
/// - `nats_url`: The URL of the NATS server to connect to.
/// - `fuel_core_config`: Configuration for the Fuel Core service, parsed using a flattened command.
#[derive(Parser)]
pub struct Cli {
    #[arg(
        long,
        value_name = "URL",
        env = "NATS_URL",
        default_value = "localhost:4222"
    )]
    nats_url: String,
    /// Flattened command structure for Fuel Core configuration.
    #[command(flatten)]
    fuel_core_config: fuel_core_bin::cli::run::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();

    let cli = Cli::parse();

    let fuel_core_service =
        fuel_core_bin::cli::run::get_service(cli.fuel_core_config).await?;

    fuel_core_service.start_and_await().await?;

    let fuel_core_database = fuel_core_service.shared.database.clone();
    let blocks_subscription = fuel_core_service
        .shared
        .block_importer
        .block_importer
        .subscribe();

    let chain_config = fuel_core_service
        .shared
        .config
        .snapshot_reader
        .chain_config();
    let chain_id = chain_config.consensus_parameters.chain_id();

    // TODO: is this still useful?
    let _base_asset_id = chain_config.consensus_parameters.base_asset_id();

    let fuel_core =
        FuelCore::new(fuel_core_database, blocks_subscription, chain_id).await;

    let nats_client_opts = NatsClientOpts::admin_opts(&cli.nats_url);
    let nats_client = NatsClient::connect(&nats_client_opts).await?;

    Publisher::new(&nats_client, fuel_core).await.run().await?;

    Ok(())
}
