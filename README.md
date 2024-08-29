<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://global.discourse-cdn.com/business6/uploads/fuel/original/2X/5/57d5a345cc15a64b636e0d56e042857f8a0e80b1.png" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Data Systems</h3>
    <p align="center">
        Official data streaming libraries and tools for the Fuel Network.
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
    </p>
    <p align="center">
        <a href="https://github.com/fuellabs/data-systems/tree/main/crates">📦 Crates</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

> [!WARNING]
> This project is currently under development and is not yet ready for production use.

Fuel Data Systems is a comprehensive suite of libraries and tools designed to enable real-time data streaming and processing from the Fuel Network. This repository houses the official data streaming ecosystem, offering developers a powerful and flexible API to interact with Fuel Network data in real-time.

With Fuel Data Systems, developers can build sophisticated applications that leverage the full potential of the Fuel Network's data, from simple block explorers to complex analytics engines and trading systems.

## 🚀 Features

-   Real-time streaming of Fuel blockchain data
-   Support for various Fuel-specific data types
-   Customizable filters for targeted data retrieval
-   Flexible delivery policies for historical and real-time data
-   Seamless integration with other Fuel ecosystem tools

## ⚡ Getting Started

1. Add `fuel-streams` to your project:

    ```sh
    cargo add fuel-streams futures tokio
    ```

2. Create a new Rust file (e.g., `src/main.rs`) with the following code to subscribe to new blocks:

    ```rust
    use fuel_streams::client::Client;
    use fuel_streams::stream::{Stream, StreamEncoder};
    use fuel_streams::blocks::Block;
    use futures::StreamExt;

    #[tokio::main]
    async fn main() -> Result<(), fuel_streams::Error> {
        let client = Client::connect("nats://stream.fuel.network").await?;
        let stream = fuel_streams::Stream::<Block>::new(&client).await;

        let mut subscription = stream.subscribe().await?;
        while let Some(message) = subscription.next().await {
            let payload = message?.payload.clone();
            let block = Block::decode(payload.into()).await;
            println!("Received block: {:?}", block);
        }

        Ok(())
    }
    ```

3. Run your project:
    ```sh
    cargo run
    ```

This example connects to the Fuel Network's NATS server and listens for new blocks. You can customize the data types or apply filters based on your specific requirements.

For advanced usage, including custom filters and delivery policies, refer to the [`fuel-streams` documentation](https://docs.rs/fuel-streams).

## 💪 Contributing

We welcome contributions to Fuel Streams! Please check our [contributing guidelines](CONTRIBUTING.md) for more information on how to get started.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](./LICENSE) for more information.
