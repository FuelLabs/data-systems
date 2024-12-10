use fuel_streams::types::FuelNetwork;
use fuel_streams_ws::{
    client::WebSocketClient,
    server::ws::models::{
        ClientMessage,
        SubscriptionPayload,
        SubscriptionType,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client =
        WebSocketClient::new(FuelNetwork::Mainnet, "admin", "admin").await?;

    client.connect()?;

    client.send_message(ClientMessage::Subscribe(SubscriptionPayload {
        topic: SubscriptionType::Stream("blocks.*.*".to_string()),
        from: None,
        to: None,
    }))?;

    let mut receiver = client.listen()?;

    while let Some(message) = receiver.recv().await {
        println!("Received: {:?}", message);
    }

    Ok(())
}
