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

## 📝 About

The `DataParser` struct provides functionality for encoding and decoding data through compression and serialization. It offers flexibility in choosing compression strategies and serialization formats, allowing for optimization of memory usage and I/O bandwidth. This utility is particularly useful when dealing with large datasets or when efficient data transfer is crucial.

## 🛠️ Usage

This library is intended for internal use within the Fuel Data Systems project. This is an example of usage outside of this crate within the project:

```rust
use fuel_data_parser::{DataEncoder, DataParser, SerializationType, DataParserError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YourDataType {
    // Your data fields here
}

impl DataEncoder for YourDataType {
    type Err = DataParserError;
}

async fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DataParser::default()
        .with_serialization_type(SerializationType::Bincode);

    // Encoding data
    let data = YourDataType { /* ... */ };
    let encoded = parser.encode(&data).await?;

    // Decoding data
    let decoded: YourDataType = parser.decode(&encoded).await?;

    Ok(())
}
```

## 🏎️ Benchmarks

To run the benchmarks and measure performance of different serialization and compression strategies:

```sh
cargo bench -p data-parser
```

> [!INFO]
> The benchmarks are located in the `../../benches` folder.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
