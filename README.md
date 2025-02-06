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

## 🚀 Features

- Real-time streaming of Fuel blockchain data
- Historical streaming of Fuel blockchain data
- Support for various Fuel-specific data types
- Customizable filters for targeted data retrieval
- Flexible deliver policies for historical and real-time data
- Seamless integration with other Fuel ecosystem tools

## 📚 Documentation

- [fuel-streams](crates/fuel-streams/README.md#fuel-streams)
    - [About The Project](crates/fuel-streams/README.md#about-the-project)
    - [Features](crates/fuel-streams/README.md#features)
    - [Installing](crates/fuel-streams/README.md#installing)
    - [Usage](crates/fuel-streams/README.md#usage)
        - [Basic Connection and Subscription](crates/fuel-streams/README.md#basic-connection-and-subscription)
        - [Subject Types and Filtering](crates/fuel-streams/README.md#subject-types-and-filtering)
        - [Multiple Subscriptions](crates/fuel-streams/README.md#multiple-subscriptions)
        - [`DeliverPolicy` Options](crates/fuel-streams/README.md#deliverpolicy-options)

## 💪 Contributing

We welcome contributions to Fuel Streams! Please check our [contributing guidelines](CONTRIBUTING.md) for more information on how to get started.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](./LICENSE) for more information.
