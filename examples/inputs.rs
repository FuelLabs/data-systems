use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// inputs from a Fuel network. It connects to a streaming service,
// subscribes to an input stream, and prints incoming inputs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Staging).await?;
    let mut connection = client.connect().await?;

    println!("Listening for inputs...");

    let subject = InputsCoinSubject::new();
    // Subscribe to the input stream with the specified configuration
    let mut stream = connection
        .subscribe::<Input>(subject, DeliverPolicy::Last)
        .await?;

    // Process incoming inputs
    while let Some(msg) = stream.next().await {
        println!("Received input: {:?}", msg.payload);
    }

    Ok(())
}
