use fuel_streams::prelude::*;
use futures::StreamExt;

// This example demonstrates how to use the fuel-streams library to stream
// UTXOs from a Fuel network. It connects to a streaming service,
// subscribes to a UTXO stream, and prints incoming UTXOs.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Mainnet).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for UTXOs...");

    // Create a subject for all UTXOs, optionally filter by type
    let subject = UtxosSubject::new().with_utxo_type(Some(UtxoType::Message)); // Example: filter for message UTXOs

    // Subscribe to the UTXO stream with the specified configuration
    let mut stream = connection
        .subscribe::<Utxo>(subject, DeliverPolicy::New)
        .await?;

    // Process incoming UTXOs
    while let Some(msg) = stream.next().await {
        println!("Received UTXO: {:?}", msg.data);
    }

    Ok(())
}
