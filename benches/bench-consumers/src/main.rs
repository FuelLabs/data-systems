use runners::runner_all::run_all_benchmarks;

mod runners;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Running benchmarks");
    run_all_benchmarks().await?;
    Ok(())
}
