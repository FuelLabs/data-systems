use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// blocks from a Fuel network. It connects to a streaming service,
// subscribes to a block stream, and prints incoming blocks.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Staging).await?;
    let mut connection = client.connect().await?;

    println!("Listening for blocks...");

    let subject = BlocksSubject::new();
    // Subscribe to the block stream with the specified configuration
    let mut stream = connection
        .subscribe::<Block>(subject, DeliverPolicy::New)
        .await?;

    // Process incoming blocks
    while let Some(block) = stream.next().await {
        println!("Received block: {:?}", block);
    }

    Ok(())
}
