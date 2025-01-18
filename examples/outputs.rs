use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// outputs from a Fuel network. It connects to a streaming service,
// subscribes to an output stream, and prints incoming outputs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for outputs...");

    let subject = OutputsCoinSubject::new();
    // Subscribe to the output stream with the specified configuration
    let mut stream = connection
        .subscribe::<Output>(subject, DeliverPolicy::New)
        .await?;

    // Process incoming outputs
    while let Some(msg) = stream.next().await {
        println!("Received output: {:?}", msg.data);
    }

    Ok(())
}
