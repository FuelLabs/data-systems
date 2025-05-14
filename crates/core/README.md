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

Fuel Streams Core is the foundation library for building data streaming applications on the Fuel blockchain. It provides a comprehensive set of tools for handling both real-time and historical blockchain data, implementing the core functionality that powers the `fuel-streams` crate.

This library includes:

- Low-level streaming infrastructure for Fuel blockchain data
- Integration with NATS for scalable message streaming
- Tools for managing subscriptions with fine-grained filtering
- Support for all Fuel-specific data types (blocks, transactions, inputs, outputs, receipts, UTXOs, etc.)
- Sophisticated subject-based filtering for targeted data retrieval

> [!NOTE]
> This crate is specifically designed for the Fuel Data Systems project, and is primarily intended for internal use. For a more user-friendly interface, consider using the [`fuel-streams`](../fuel-streams) crate instead.

## 🛠️ Installing

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-streams-core = "0.1.0"  # Use the latest version available
```

## 🚀 Features

The `fuel-streams-core` crate provides several key features:

- **Robust Streaming Engine**: Built on NATS for reliable, high-performance messaging
- **Multiple Data Types**: Full support for all Fuel blockchain data types
- **Fine-grained Filtering**: Advanced filtering capabilities through a rich subject system
- **Efficient Data Handling**: Optimized for performance with Fuel blockchain data
- **Historical Data Access**: Support for retrieving historical data through the Deliver Policy system
- **User Authentication**: Integration with API key-based authentication for secure access

## 📊 Usage

### Basic Connection and Stream Creation

```rust,no_run
use fuel_streams_core::prelude::*;
use fuel_streams_domains::infra::*;
use fuel_web_utils::api_key::*;
use fuel_message_broker::*;
use futures::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to NATS server and database
    let db = Db::new(DbConnectionOpts::default()).await?;
    let broker = Arc::new(NatsMessageBroker::setup("nats://localhost:4222", None).await?);
    let db = Arc::new(db);

    // Create a stream for blocks
    let stream = Stream::<Block>::get_or_init(&broker, &db).await;

    // Create the subscription subject (blocks.*.*) with role-based access control
    let subject = BlocksSubject::new();
    let api_key_role = ApiKeyRole::default();

    // Subscribe to the stream with the chosen delivery policy
    let mut subscription = stream
        .subscribe(subject, DeliverPolicy::New, &api_key_role)
        .await;

    // Process incoming blocks
    while let Some(result) = subscription.next().await {
        match result {
            Ok(response) => println!("Received block: {:?}", response),
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }

    Ok(())
}
```

### Using the FuelStreams High-Level API

The `FuelStreams` struct provides a high-level API for working with all data types:

```rust,no_run
use fuel_streams_core::prelude::*;
use fuel_streams_domains::infra::*;
use fuel_web_utils::api_key::*;
use fuel_message_broker::*;
use futures::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup broker and database
    let db = Arc::new(Db::new(DbConnectionOpts::default()).await?);
    let broker = Arc::new(NatsMessageBroker::setup("nats://localhost:4222", None).await?);

    // Create the FuelStreams instance with all stream types
    let fuel_streams = FuelStreams::new(&broker, &db).await;

    // Create a subscription with dynamic subject
    let subscription = Subscription {
        payload: TransactionsSubject::new().into(),
        deliver_policy: DeliverPolicy::New,
    };

    // Subscribe using the API key role for authentication
    let api_key_role = ApiKeyRole::default();
    let mut stream = fuel_streams
        .subscribe_by_subject(&api_key_role, &subscription)
        .await?;

    // Process incoming data
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => println!("Received data: {:?}", response),
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }

    Ok(())
}
```

### Publishing Data

```rust,no_run
use fuel_streams_core::prelude::*;
use fuel_streams_domains::infra::*;
use fuel_message_broker::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup connections
    let db = Arc::new(Db::new(DbConnectionOpts::default()).await?);
    let broker = Arc::new(NatsMessageBroker::setup("nats://localhost:4222", None).await?);

    // Create specific stream for blocks
    let block_stream = Stream::<Block>::get_or_init(&broker, &db).await;

    // Create data to publish
    let block_data = Block::default(); // Your block data here
    let response = StreamResponse::new(block_data);

    // Publish to the stream
    let subject = "blocks.100.hash";
    block_stream.publish(subject, &Arc::new(response)).await?;

    Ok(())
}
```

## 🔧 DeliverPolicy Options

The `DeliverPolicy` enum provides control over how messages are delivered in subscriptions:

- `New`: Delivers only new messages that arrive after subscription
- `FromHeight(BlockHeight)`: Delivers messages starting from a specific block height
- `FromBlock { block_height }`: Delivers messages starting from a specific block
- `FromFirst`: Delivers all messages starting from the first available message

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
