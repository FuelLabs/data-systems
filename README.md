<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
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

Fuel Data Systems is a comprehensive suite of libraries and tools designed to enable real-time and historical data streaming and processing from the Fuel Network. This repository houses the official data streaming ecosystem, offering developers a powerful and flexible API to interact with Fuel Network data in real-time.

With Fuel Data Systems, developers can build sophisticated applications that leverage the full potential of the Fuel Network's data, from simple block explorers to complex analytics engines and trading systems.

### Getting Started

To get started with local development and syncing blocks locally, see the [Syncing Blocks Locally](services/publisher/README.md#syncing-blocks-locally) section in the Publisher README.

The [CONTRIBUTING.md](CONTRIBUTING.md) file contains detailed information about setting up your development environment and contributing to this project.

## 🚀 Features

- Real-time streaming of Fuel blockchain data
- Historical streaming of Fuel blockchain data
- Support for various Fuel-specific data types
- Customizable filters for targeted data retrieval
- Flexible deliver policies for historical and real-time data
- Seamless integration with other Fuel ecosystem tools

## 📚 Documentation

To check this stream service up and running, visit:

- [https://stream.fuel.network](https://stream.fuel.network)

For the REST API documentation (Mainnet), visit:

- [https://api-rest-mainnet.fuel.network/swagger-ui](https://api-rest-mainnet.fuel.network/swagger-ui)

For codebase documentation, see the README files in the relevant directories:

- [Crates Documentation](crates/)
- [Services Documentation](services/)
- [Cluster Documentation](cluster/)

## 💻 Fuel Stream Rust SDK

The [fuel-streams](crates/fuel-streams/README.md) library provides a simple Rust SDK for connecting to and consuming data from the Fuel blockchain.

### Installing

```sh
cargo add fuel-streams futures tokio
```

### Basic Usage Example

```rust
use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client and establish connection
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    println!("Listening for blocks...");

    // Choose which subjects you want to filter
    let subjects = vec![BlocksSubject::new().into()];

    // Subscribe to blocks with new deliver policy
    let mut stream = connection
        .subscribe(subjects, DeliverPolicy::New)
        .await?;

    while let Some(block) = stream.next().await {
        println!("Received block: {:?}", block);
    }

    Ok(())
}
```

### Filtering Transactions

```rust
use fuel_streams::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(FuelNetwork::Local).with_api_key("your_key");
    let mut connection = client.connect().await?;

    // Create a subject for script transactions
    let subjects = vec![
        TransactionsSubject::new()
            .with_tx_type(Some(TransactionType::Script))
            .into()
    ];

    // Subscribe to the filtered transaction stream
    let mut stream = connection
        .subscribe(subjects, DeliverPolicy::New)
        .await?;

    while let Some(transaction) = stream.next().await {
        println!("Received transaction: {:?}", transaction);
    }

    Ok(())
}
```

For more examples and detailed documentation, see the [fuel-streams crate documentation](https://docs.rs/fuel-streams/).

## 📑 Architecture Components

### Services

| Service                                           | Description                                                      |
| ------------------------------------------------- | ---------------------------------------------------------------- |
| [API Service](services/api/README.md)             | REST API for retrieving blockchain data from an indexed database |
| [Consumer Service](services/consumer/README.md)   | Processes and stores blockchain data in a database               |
| [Publisher Service](services/publisher/README.md) | Subscribes to new blocks and publishes them to a message broker  |
| [WebSocket Service](services/webserver/README.md) | Real-time data streaming via WebSocket connections               |
| [Dune Service](services/dune/README.md)           | Processes blockchain data for analytics with Dune                |

### Libraries

| Library                                           | Description                                                   |
| ------------------------------------------------- | ------------------------------------------------------------- |
| [fuel-streams](crates/fuel-streams/README.md)     | Main library for streaming Fuel blockchain data               |
| [web-utils](crates/web-utils/README.md)           | Web utilities for building web services in the Fuel ecosystem |
| [domains](crates/domains/README.md)               | Domain models and database infrastructure                     |
| [subject](crates/subject/README.md)               | Subject derive macro for type-safe subject definitions        |
| [types](crates/types/README.md)                   | Core type definitions and utilities                           |
| [core](crates/core/README.md)                     | Core functionalities and shared components                    |
| [data-parser](crates/data-parser/README.md)       | Parser for Fuel blockchain data                               |
| [message-broker](crates/message-broker/README.md) | Message broker implementation for event publishing            |

### Deployment and Infrastructure

| Component                          | Description                                    |
| ---------------------------------- | ---------------------------------------------- |
| [Cluster](cluster/README.md)       | Deployment infrastructure and configuration    |
| [Docker](cluster/docker/README.md) | Docker configuration for local development     |
| [Charts](cluster/charts/README.md) | Helm charts for Kubernetes deployment          |
| [Scripts](scripts/README.md)       | Utility scripts for development and deployment |

## 🛠️ Development

For local development:

1. **Setup Environment**:

    ```bash
    make create-env
    make setup
    ```

2. **Start Required Services**:

    ```bash
    make start-docker
    make setup-db
    ```

3. **Run Services**:
    - API Service: `make run-api`
    - Publisher Service: `make run-publisher`
    - Consumer Service: `make run-consumer`
    - WebSocket Service: `make run-webserver`
    - Dune Service: `make run-dune`

See the [CONTRIBUTING.md](CONTRIBUTING.md) for more detailed development instructions.

## 💪 Contributing

We welcome contributions to Fuel Streams! Please check our [contributing guidelines](CONTRIBUTING.md) for more information on how to get started.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](./LICENSE) for more information.
