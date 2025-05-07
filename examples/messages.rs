use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ⚠️ Please change here for FuelNetwork::Mainnet if you want to connect to the mainnet
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for messages...");

    // Create a subject for all messages of a specific type
    let subject =
        MessagesSubject::new().with_message_type(Some(MessageType::Imported)); // Example: filter for imported messages
    let filter_subjects = vec![subject.into()];

    // Subscribe to the message stream with the specified configuration
    let mut stream = connection
        .subscribe(filter_subjects, DeliverPolicy::New)
        .await?;

    // Process incoming messages
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let message = msg.payload.as_message()?;
        println!("Received message: {:?}", message);
    }

    Ok(())
}
