<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Core</h3>
    <p align="center">
        The core library for data streaming in the Fuel Data Systems project.
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-core" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-core">
        </a>
        <a href="https://crates.io/crates/fuel-streams-core" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-core?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-core/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-core/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-core">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Streams Core is a library for building data streaming applications on the Fuel blockchain. It provides tools for efficient handling of real-time blockchain data, using NATS for scalable streaming and offering support for Fuel-specific data types.

> [!NOTE]
> This crate is specifically modeled for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🛠️ Installing

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-streams-core = "*"
```

## 🚀 Usage

Here's a simple example to get you started with Fuel Streams Core:

```rust,no_run
use fuel_streams_core::prelude::*;
use fuel_streams_store::db::*;
use fuel_message_broker::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to NATS server
    let db = Db::new(DbConnectionOpts::default()).await?;
    let broker = NatsMessageBroker::setup("nats://localhost:4222", None).await?;

    // Create or get existing stream for blocks
    let stream = Stream::<Block>::get_or_init(&broker, &db.arc()).await;

    // Subscribe to the stream
    let subject = BlocksSubject::new(); // blocks.*.*
    let mut subscription = stream.subscribe(subject, DeliverPolicy::New).await;

    // Process incoming blocks
    while let Some(block) = subscription.next().await {
        println!("Received block: {:?}", block?);
    }

    Ok(())
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
