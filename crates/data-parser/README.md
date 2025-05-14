<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Data Parser</h3>
    <p align="center">
        A utility library for encoding and decoding data in the Fuel Data Systems project.
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-data-parser" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-data-parser">
        </a>
        <a href="https://crates.io/crates/fuel-data-parser" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-data-parser?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-data-parser/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-data-parser/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-data-parser">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

The Fuel Data Parser is a specialized utility library that provides functionality for efficient encoding and decoding of data within the Fuel Data Systems project. It offers a consistent interface for serialization and deserialization operations, focusing on performance and reliability when handling Fuel blockchain data.

This library provides:

- A unified interface for serialization/deserialization of data structures
- Support for JSON serialization with potential for expansion to other formats
- A trait-based system for easy implementation on custom data types
- Error handling tailored for data parsing operations

> [!NOTE]
> This crate is primarily designed for internal use within the Fuel Data Systems project, serving as a foundational utility for other components that need to encode or decode data.

## 🛠️ Installing

Add this dependency to your `Cargo.toml`:

```toml
[dependencies]
fuel-data-parser = "0.1.0"  # Use the latest version available
```

## 🚀 Features

The `fuel-data-parser` crate provides several key features:

- **Consistent API**: Unified interface for all serialization/deserialization operations
- **JSON Support**: Built-in support for JSON encoding and decoding
- **Error Handling**: Comprehensive error types for debugging serialization issues
- **Extensible Design**: Architecture that allows for additional serialization formats
- **Trait-Based System**: Easy implementation on custom data types through the `DataEncoder` trait

## 📊 Usage

### Basic Usage

Here's a basic example of using the `DataParser` to encode and decode data:

```rust
use fuel_data_parser::{DataEncoder, DataParser, SerializationType};
use serde::{Serialize, Deserialize};

// Define a data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserData {
    id: u64,
    name: String,
    active: bool,
}

// Implement the DataEncoder trait
impl DataEncoder for UserData {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a DataParser with the default configuration (JSON)
    let parser = DataParser::default();

    // Create some data
    let user = UserData {
        id: 1,
        name: "Alice".to_string(),
        active: true,
    };

    // Encode the data to JSON
    let encoded_data = parser.encode_json(&user)?;
    println!("Encoded data size: {} bytes", encoded_data.len());

    // Decode the data back to the original type
    let decoded_user: UserData = parser.decode_json(&encoded_data)?;
    println!("Decoded user: {:?}", decoded_user);

    // Verify equality
    assert_eq!(user, decoded_user);

    Ok(())
}
```

### Using the DataEncoder Trait

The `DataEncoder` trait provides convenience methods for common operations:

```rust
use fuel_data_parser::{DataEncoder, DataParser};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockData {
    height: u64,
    hash: String,
    timestamp: u64,
}

// Implement DataEncoder to get encode/decode methods
impl DataEncoder for BlockData {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let block = BlockData {
        height: 100,
        hash: "0x123abc...".to_string(),
        timestamp: 1625097600,
    };

    // Use trait methods directly on the data
    let encoded = block.encode_json()?;

    // Use static methods for decoding
    let decoded = BlockData::decode_json(&encoded)?;

    // Convert to JSON value for custom processing
    let json_value = block.to_json_value()?;
    println!("Block JSON: {}", json_value);

    Ok(())
}
```

### Using in Async Contexts

The `DataParser` can be used in async code:

```rust
use fuel_data_parser::{DataEncoder, DataParser};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkMessage {
    message_type: String,
    payload: Vec<u8>,
}

impl DataEncoder for NetworkMessage {}

async fn process_message(message: &NetworkMessage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let parser = DataParser::default();

    // In a real application, this might involve network operations
    let encoded = parser.encode_json(message)?;

    Ok(encoded)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = NetworkMessage {
        message_type: "data".to_string(),
        payload: vec![1, 2, 3, 4],
    };

    let encoded = process_message(&message).await?;
    println!("Encoded message size: {} bytes", encoded.len());

    Ok(())
}
```

## 🏎️ Benchmarks

To run the benchmarks and measure performance of the serialization operations:

```sh
cargo bench -p fuel-data-parser
```

The benchmarks compare different operations and can help you understand the performance characteristics of the library.

> [!INFO]
> The benchmarks are located in the `../../benches` directory of the repository.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
