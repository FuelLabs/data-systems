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
    service.start()?;

    let subscription = service.shared.block_importer.block_importer.subscribe();

    fuel_core_nats::nats_publisher(subscription, cli.nats_url).await?;

    Ok(())
}
