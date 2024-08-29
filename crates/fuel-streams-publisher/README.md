<br/>
<div align="center">
    <a href="https://github.com/fuellabs/data-systems">
        <img src="https://global.discourse-cdn.com/business6/uploads/fuel/original/2X/5/57d5a345cc15a64b636e0d56e042857f8a0e80b1.png" alt="Logo" width="80" height="80">
    </a>
    <h3 align="center">Fuel Streams Publisher</h3>
    <p align="center">
        A binary that subscribes to events from a Fuel client or node and publishes streams consumable via the fuel-streams SDK
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
        <a href="https://github.com/fuellabs/data-systems/tree/main/crates/fuel-streams-publisher">📚 Documentation</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=bug&template=bug-report---.md">🐛 Report Bug</a>
        <span>&nbsp;</span>
        <a href="https://github.com/fuellabs/data-systems/issues/new?labels=enhancement&template=feature-request---.md">✨ Request Feature</a>
    </p>
</div>

## 📝 About The Project

The Fuel Streams Publisher is a binary that subscribes to events emitted from a Fuel client or node and publishes streams that can be consumed via the `fuel-streams` SDK.

## ⚡️ Getting Started

### Prerequisites

-   [Rust toolchain](https://www.rust-lang.org/tools/install)
-   [Docker](https://www.docker.com/get-started/) (optional)

### Development

1. Generate the `KEYPAIR` environment variable:

    ```sh
    fuel-core-keygen new --key-type peering -p
    ```

2. Generate an `INFURA_API_KEY` from [Infura](https://app.infura.io/)

3. Copy `.env.sample` to `.env` and update the `KEYPAIR` and `INFURA_API_KEY` with the values generated above

4. Run the binary:

    - From the monorepo's root:

        ```sh
        ./scripts/start-publisher.sh
        ```

    - Or using `make` and `docker`:

        ```sh
        make start/publisher
        ```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For more information on contributing, please see the [CONTRIBUTING.md](../../CONTRIBUTING.md) file in the root of the repository.

## 📜 License

This project is licensed under the `Apache-2.0` license. See [`LICENSE`](../../LICENSE) for more information.
