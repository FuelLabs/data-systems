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
    #[arg(long, value_name = "NKEY", env = "NATS_NKEY")]
    nats_nkey: Option<String>,
    #[command(flatten)]
    fuel_core_config: fuel_core_bin::cli::run::Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();

    let cli = Cli::parse();
    let fuel_core = fuel_core_bin::cli::run::get_service(cli.fuel_core_config.clone())?;
    fuel_core.start()?;
    run_publisher(&cli.nats_url, cli.nats_nkey, &fuel_core).await?;

    Ok(())
}

trait FuelCore {
    fn get_chain_id_and_base_asset_id(
        &self,
    ) -> (
        fuel_core_types::fuel_types::ChainId,
        fuel_core_types::fuel_types::AssetId,
    );
    fn get_database(&self) -> fuel_core::combined_database::CombinedDatabase;
    fn get_blocks_subscription(
        &self,
    ) -> tokio::sync::broadcast::Receiver<
        std::sync::Arc<
            dyn std::ops::Deref<
                    Target = fuel_core_types::services::block_importer::ImportResult,
                > + Send
                + Sync,
        >,
    >;
}

impl FuelCore for fuel_core_bin::FuelService {
    fn get_chain_id_and_base_asset_id(
        &self,
    ) -> (
        fuel_core_types::fuel_types::ChainId,
        fuel_core_types::fuel_types::AssetId,
    ) {
        let chain_config = self.shared.config.snapshot_reader.chain_config();

        (
            chain_config.consensus_parameters.chain_id(),
            *chain_config.consensus_parameters.base_asset_id(),
        )
    }

    fn get_database(&self) -> fuel_core::combined_database::CombinedDatabase {
        self.shared.database.clone()
    }

    fn get_blocks_subscription(
        &self,
    ) -> tokio::sync::broadcast::Receiver<
        std::sync::Arc<
            dyn std::ops::Deref<
                    Target = fuel_core_types::services::block_importer::ImportResult,
                > + Send
                + Sync,
        >,
    > {
        self.shared.block_importer.block_importer.subscribe()
    }
}

async fn run_publisher(
    nats_url: &str,
    nats_nkey: Option<String>,
    fuel_core: &impl FuelCore,
) -> anyhow::Result<()> {
    let (chain_id, base_asset_id) = fuel_core.get_chain_id_and_base_asset_id();

    let publisher = fuel_core_nats::Publisher::new(
        nats_url,
        nats_nkey,
        chain_id,
        base_asset_id,
        fuel_core.get_database(),
        fuel_core.get_blocks_subscription(),
    )
    .await?;
    publisher.run().await?;

    Ok(())
}
