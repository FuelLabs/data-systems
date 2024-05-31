use clap::Parser;

use fuel_core_bin::cli::run::{get_service, Command};
use fuel_core_services::Service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fuel_core_bin::cli::init_logging();
    
    let command = Command::parse();
    let service = get_service(command)?;
    service.start()?;

    let subscription = service.shared.block_importer.block_importer.subscribe();

    fuel_core_nats::nats_publisher(subscription).await?;

    Ok(())
}