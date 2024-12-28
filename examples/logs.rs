use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// logs from a Fuel network. It connects to a streaming service,
// subscribes to a log stream, and prints incoming logs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Staging).await?;
    let mut connection = client.connect().await?;

    println!("Listening for logs...");

    let subject = LogsSubject::new();
    // Subscribe to the log stream with the specified configuration
    let mut stream = connection
        .subscribe::<Log>(subject, DeliverPolicy::Last)
        .await?;

    // Process incoming logs
    while let Some(msg) = stream.next().await {
        println!("Received log: {:?}", msg.payload);
    }

    Ok(())
}
