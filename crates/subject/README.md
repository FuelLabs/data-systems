<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://fuellabs.notion.site/image/https%3A%2F%2Fprod-files-secure.s3.us-west-2.amazonaws.com%2F9ff3607d-8974-46e8-8373-e2c96344d6ff%2F81a0a0d9-f3c7-4ccb-8af5-40ca8a4140f9%2FFUEL_Symbol_Circle_Green_RGB.png?table=block&id=cb8fc88a-4fc3-4f28-a974-9c318a65a2c6&spaceId=9ff3607d-8974-46e8-8373-e2c96344d6ff&width=2000&userId=&cache=v2" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Subject</h3>
    <p align="center">
        Macros for implementing subject functionality in the fuel-streams ecosystem
    </p>
    <p align="center">
        <a href="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml" style="text-decoration: none;">
            <img src="https://github.com/FuelLabs/data-systems/actions/workflows/ci.yaml/badge.svg?branch=main" alt="CI">
        </a>
        <a href="https://codecov.io/gh/FuelLabs/data-systems" style="text-decoration: none;">
            <img src="https://codecov.io/gh/FuelLabs/data-systems/graph/badge.svg?token=1zna00scwj" alt="Coverage">
        </a>
        <a href="https://crates.io/crates/fuel-streams-subject" style="text-decoration: none;">
            <img alt="Crates.io MSRV" src="https://img.shields.io/crates/msrv/fuel-streams-subject">
        </a>
        <a href="https://crates.io/crates/fuel-streams-subject" style="text-decoration: none;">
            <img src="https://img.shields.io/crates/v/fuel-streams-subject?label=latest" alt="crates.io">
        </a>
        <a href="https://docs.rs/fuel-streams-subject/" style="text-decoration: none;">
            <img src="https://docs.rs/fuel-streams-subject/badge.svg" alt="docs">
        </a>
    </p>
    <p align="center">
        <a href="https://docs.rs/fuel-streams-subject/">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

# 📝 About The Project

Provides macros for implementing subject functionality in the fuel-streams ecosystem.

> [!NOTE]
> This crate is specifically modeled for the Fuel Data Systems project, and is not intended for general use outside of the project.

## 🚀 Usage

The `Subject` derive macro allows you to easily implement the `Subject` trait for your structs. It generates methods for parsing, building, and creating subjectsfor your subject.

Example:

```rust
use fuel_streams_subject::subject::*;

#[derive(Subject, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[subject(id = "test")]
#[subject(entity = "Test")]
#[subject(query_all = "test.>")]
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

// Create a subject string
assert_eq!(TestSubject::build_string(None, Some(10)), "test.*.10");

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
