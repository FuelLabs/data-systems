<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://global.discourse-cdn.com/business6/uploads/fuel/original/2X/5/57d5a345cc15a64b636e0d56e042857f8a0e80b1.png" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Macros</h3>
    <p align="center">
        Macros for implementing traits and deriving functionality in the fuel-streams ecosystem
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-macros" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-macros">
        </a>
        <a href="https://crates.io/crates/fuel-streams-macros" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-macros?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-macros/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-macros/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-macros/">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

# 📝 About The Project

Provides macros for implementing traits and deriving functionality in the fuel-streams ecosystem.

> [!NOTE]
> This crate is specifically modeled for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🚀 Usage

The `Subject` derive macro allows you to easily implement the `Subject` trait for your structs. It generates methods for parsing, building, and creating wildcards for your subject.

Example:

```rust
use fuel_streams_macros::subject::*;

#[derive(Subject, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[subject(id = "test")]
#[subject(entity = "Test")]
#[subject(wildcard = "test.>")]
#[subject(format = "test.{field1}.{field2}")]
struct TestSubject {
    field1: Option<String>,
    field2: Option<u32>,
}

// Create a new TestSubject
let subject = TestSubject {
    field1: Some("foo".to_string()),
    field2: Some(55),
};

// Parse the subject
assert_eq!(subject.parse(), "test.foo.55");

// Create a wildcard
assert_eq!(TestSubject::wildcard(None, Some(10)), "test.*.10");

// Create using the build method
let subject = TestSubject::build(Some("foo".into()), Some(55));
assert_eq!(subject.parse(), "test.foo.55");

// Create a new TestSubject with the builder pattern
let subject = TestSubject::new()
    .with_field1(Some("foo".to_string()))
    .with_field2(Some(55));
assert_eq!(subject.parse(), "test.foo.55");

// Convert to a string
assert_eq!(&subject.to_string(), "test.foo.55");
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This repo is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
