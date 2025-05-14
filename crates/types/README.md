<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Types</h3>
    <p align="center">
        Core types and utilities for the Fuel Data Systems project
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-types" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-types">
        </a>
        <a href="https://crates.io/crates/fuel-streams-types" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-types?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-types/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-types/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-types">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

Fuel Streams Types provides the core type definitions and utilities for the Fuel Data Systems project. It serves as the foundation for type-safe interactions with the Fuel blockchain, offering a comprehensive set of primitive types, macros, and integrations with the Fuel Core.

> [!NOTE]
> This crate is specifically designed for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🚀 Features

- **Type-Safe Primitives**: Strongly-typed wrappers for blockchain data types
- **Fuel Core Integration**: Seamless interaction with Fuel Core services
- **Code Generation Macros**: Utilities for generating common type implementations
- **Database Compatibility**: SQLx integration for database operations
- **Serialization Support**: Serde implementations for all types
- **OpenAPI Documentation**: Utoipa integration for API documentation
- **Avro Schema Generation**: Support for Apache Avro data serialization
- **Testing Utilities**: Helpers for unit and integration testing

## 🏗️ Architecture

The crate is organized into the following main components:

### Primitives

Core blockchain data types with strong typing and validation:

- **BlockHeight**: Blockchain block heights
- **BlockTimestamp**: Timestamps for blocks
- **Address**: Account and contract addresses
- **TxType**: Transaction types
- **InputType**: Transaction input types
- **OutputType**: Transaction output types
- **ReceiptType**: Transaction receipt types
- **TxStatus**: Transaction status indicators
- **UtxoId**: UTXO identifiers
- **And many more**: Comprehensive coverage of all Fuel blockchain primitives

### Macros

Code generation utilities for common patterns:

- **wrapped_int**: Strongly-typed integer wrappers
- **wrapper_str**: Strongly-typed string wrappers
- **gen_bytes**: Byte array wrapper generation
- **enum_str**: String-based enum utilities
- **avro**: Apache Avro schema generation
- **open_api**: OpenAPI schema generation

### Fuel Core Integration

Interfaces and types for interacting with Fuel Core:

- **FuelCoreLike**: Trait for Fuel Core service interactions
- **FuelCore**: Implementation of the FuelCoreLike trait
- **Type Re-exports**: Convenient access to Fuel Core types

## 🛠️ Usage

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-streams-types = "*"
```

### Using Primitive Types

```rust
use fuel_streams_types::{BlockHeight, Address, Bytes32};

// Create a block height
let height = BlockHeight::from(123u32);
assert_eq!(height.into_inner(), 123);

// Create an address from a hex string
let address_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
let address = Address::try_from(address_str).unwrap();

// Create a byte array
let bytes = Bytes32::random();
```

### Using Integer Wrapper Macros

```rust
use fuel_streams_types::{declare_integer_wrapper, impl_avro_schema_for_wrapped_int};

// Define a new integer wrapper type
declare_integer_wrapper!(TransactionIndex, u32);

// Add Avro schema support
impl_avro_schema_for_wrapped_int!(TransactionIndex, u32);

// Use the new type
let tx_index = TransactionIndex::from(42u32);
assert_eq!(tx_index.into_inner(), 42);
```

### Interacting with Fuel Core

```rust
use fuel_streams_types::{FuelCore, FuelCoreLike};
use fuel_core_bin::cli::run::Command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a Fuel Core command
    let command = Command::default();

    // Initialize Fuel Core
    let fuel_core = FuelCore::new(command).await?;

    // Start the service
    fuel_core.start().await?;

    // Get the latest block height
    let height = fuel_core.get_latest_block_height()?;
    println!("Latest block height: {}", height);

    // Stop the service
    fuel_core.stop().await;

    Ok(())
}
```

### Working with Timestamps

```rust
use fuel_streams_types::BlockTimestamp;
use chrono::{DateTime, Utc};

// Create a timestamp from the current time
let now = BlockTimestamp::now();

// Convert to a DateTime<Utc>
let datetime: DateTime<Utc> = now.into();

// Create from a Unix timestamp
let timestamp = BlockTimestamp::from_unix_timestamp(1625097600);
```

## 🧪 Testing

The crate provides test helpers for working with Fuel types:

```rust
use fuel_streams_types::{BlockHeight, Address};

#[test]
fn test_block_height() {
    // Generate a random block height
    let height = BlockHeight::random();

    // Generate a random height with a maximum value
    let height_with_max = BlockHeight::random_max(1000);
    assert!(height_with_max.into_inner() < 1000);
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
