use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// outputs from a Fuel network. It connects to a streaming service,
// subscribes to an output stream, and prints incoming outputs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Mainnet).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for outputs...");

    let subject = OutputsCoinSubject::new();
    let filter_subjects = vec![subject.into()];

    // Subscribe to the output stream with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::New)
        .await?;

    // Process incoming outputs
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let output = msg.payload.as_output()?;
        println!("Received output: {:?}", output);
    }

    Ok(())
}
