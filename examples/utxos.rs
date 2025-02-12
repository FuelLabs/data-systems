use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ⚠️ Please change here for FuelNetwork::Mainnet if you want to connect to the mainnet
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for UTXOs...");

    // Create a subject for all UTXOs, optionally filter by type
    let subject = UtxosSubject::new().with_utxo_type(Some(UtxoType::Message)); // Example: filter for message UTXOs
    let filter_subjects = vec![subject.into()];

    // Subscribe to the UTXO stream with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::New)
        .await?;

    // Process incoming UTXOs
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let utxo = msg.payload.as_utxo()?;
        println!("Received UTXO: {:?}", utxo);
    }

    Ok(())
}
