<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Domains</h3>
    <p align="center">
        Domain models and database infrastructure for the Fuel Data Systems project
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-domains" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-domains">
        </a>
        <a href="https://crates.io/crates/fuel-streams-domains" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-domains?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-domains/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-domains/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-domains">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Streams Domains provides the core domain models and database infrastructure for the Fuel Data Systems project. It defines the data structures, repositories, and database schema for all blockchain entities, enabling efficient storage, retrieval, and manipulation of blockchain data.

> [!NOTE]
> This crate is specifically modeled for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🏗️ Architecture

The crate is organized around domain-driven design principles, with each blockchain entity (blocks, transactions, etc.) represented as a separate domain module. Each domain module contains:

- **Types**: Core data structures representing the domain entity
- **Repository**: Database access layer for the domain entity
- **DB Item**: Database representation of the entity
- **Query Parameters**: Structured query parameters for filtering entities
- **Subjects**: Message subjects for pub/sub communication
- **Packets**: Serialization formats for network communication

The `infra` module provides shared infrastructure components:

- **DB**: Database connection and transaction management
- **Record**: Common record handling functionality
- **Repository**: Base repository patterns and traits

## 📊 Domain Entities

The following domain entities are supported:

- **Blocks**: Blockchain blocks with headers and metadata
- **Transactions**: Blockchain transactions
- **Inputs**: Transaction inputs
- **Outputs**: Transaction outputs
- **Messages**: Cross-chain messages
- **Receipts**: Transaction execution receipts
- **UTXOs**: Unspent transaction outputs
- **Predicates**: Smart contract predicates

## 🗃️ Database Schema

The crate includes SQL migrations for creating and managing the database schema. These migrations define tables for each domain entity and establish relationships between them.

## 🛠️ Usage

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-streams-domains = "*"
```

### Working with Database Connection

```rust
use std::sync::Arc;
use fuel_streams_domains::infra::{Db, DbConnectionOpts};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a database connection
    let db_opts = DbConnectionOpts::default();
    let db = Db::new(db_opts).await?;

    // Use the database connection
    // ...

    Ok(())
}
```

### Finding Blocks

```rust
use std::sync::Arc;
use fuel_streams_domains::{
    blocks::{Block, BlocksQuery},
    infra::{Db, DbConnectionOpts, QueryOptions, Repository},
};
use fuel_streams_types::BlockHeight;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a database connection
    let db_opts = DbConnectionOpts::default();
    let db = Db::new(db_opts).await?;

    // Find the last block height
    let options = QueryOptions::default();
    let last_height = Block::find_last_block_height(&db, &options).await?;
    println!("Last block height: {}", last_height);

    // Find blocks in a specific height range
    let start_height = BlockHeight::from(100);
    let end_height = BlockHeight::from(200);
    let blocks = Block::find_in_height_range(&db.pool, start_height, end_height, &options).await?;
    println!("Found {} blocks in range", blocks.len());

    Ok(())
}
```

### Working with Transactions

```rust
use std::sync::Arc;
use fuel_streams_domains::{
    blocks::Block,
    infra::{Db, DbConnectionOpts, Repository},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a database connection
    let db_opts = DbConnectionOpts::default();
    let db = Db::new(db_opts).await?;

    // Create a block (simplified example)
    let block = Block::default();

    // Get transactions for a block
    let transactions = block.transactions_from_block(&db.pool).await?;
    println!("Block has {} transactions", transactions.len());

    Ok(())
}
```

### Finding Blocks with Transactions

```rust
use std::sync::Arc;
use fuel_streams_domains::{
    blocks::Block,
    infra::{Db, DbConnectionOpts, QueryOptions, Repository},
};
use fuel_streams_types::BlockHeight;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a database connection
    let db_opts = DbConnectionOpts::default();
    let db = Db::new(db_opts).await?;

    // Find blocks with their transactions in a specific height range
    let options = QueryOptions::default();
    let start_height = BlockHeight::from(100);
    let end_height = BlockHeight::from(110);

    let blocks_with_txs = Block::find_blocks_with_transactions(
        &db.pool,
        start_height,
        end_height,
        &options
    ).await?;

    // Process blocks and their transactions
    for (block, transactions) in blocks_with_txs {
        println!("Block #{} has {} transactions", block.block_height, transactions.len());
    }

    Ok(())
}
```

## 🧪 Testing

The crate provides mock implementations of domain entities for testing purposes:

```rust
use fuel_streams_domains::mocks::*;

#[test]
fn test_with_mock_data() {
    let mock_block = MockBlock::default();
    let mock_tx = MockTransaction::default();

    // Use mock data for testing
    assert_eq!(mock_block.height.value(), 0);
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
