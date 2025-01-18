use clap::Parser;
use load_tester::runners::{cli::Cli, runner_all::LoadTesterEngine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("Running load test ...");
    let load_tester = LoadTesterEngine::new(
        cli.network,
        cli.api_key,
        cli.max_subscriptions,
        cli.step_size,
    );
    load_tester.run().await?;
    println!("Finished load testing!");
    Ok(())
}
