use fuel_streams::prelude::*;
use futures::StreamExt;

const TX_ID: &str =
    "0x243ef4c2301f44eecbeaf1c39fee9379664b59a2e5b75317e8c7e7f26a25ed4d";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a client connection to the Fuel streaming service
    let mut client = Client::new(FuelNetwork::Mainnet).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for receipts...");

    // Create a subject for all receipt types related to the contract
    let subject = ReceiptsReturnSubject::new().with_tx_id(Some(TX_ID.into()));

    // Subscribe to the receipt stream with the specified configuration
    let mut stream = connection.subscribe(subject, DeliverPolicy::New).await?;

    // Process incoming receipts
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let receipt = msg.payload.as_receipt()?;
        println!("Received receipt: {:?}", receipt);
    }

    Ok(())
}
