# Contributing

This guide will show you how to run this project locally if you want to test or contribute to it.

## 🛠 Prerequisites

Most projects under the umbrella of data systems are written in Rust, so we prefer using Rust tooling and community standards. Ensure you have the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install) (version 1.84.0)
- [Rust Nightly](https://rust-lang.github.io/rustup/concepts/channels.html) (version nightly-2025-01-24)
- [NodeJS](https://nodejs.org/en/download/) (version >=22.11.0)
- [PNPM](https://pnpm.io/installation) (version 10.0.0)
- [Make](https://www.gnu.org/software/make/)
- [Pre-commit](https://pre-commit.com/#install)
- [NATS](https://nats.io/download/)
- [Tilt](https://docs.tilt.dev/install.html)
- [Minikube](https://minikube.sigs.k8s.io/docs/start/)
- [Kubernetes](https://kubernetes.io/)
- [Python3](https://www.python.org/downloads/)
- [Docker](https://www.docker.com/get-started)

## 📟 Setting up

First, clone this repository:

```sh
git clone git@github.com:fuellabs/data-systems.git
cd data-systems
```

Now, install the necessary tools to ensure code quality and standards. Use Make to simplify this process:

```sh
make setup
```

After setup, you'll need to create the environment configuration. First, make sure you have an Infura API key:

1. Go to [Infura](https://infura.io/) and create an account
2. Create a new project
3. Copy your project ID (API key)

Then run the environment setup command:

```sh
make create-env
```

The script will prompt you to enter your Infura API key and will automatically:

- Generate a new keypair for P2P communication
- Create a `.env` file from the template
- Configure the environment with your Infura key and the generated keypair

You can check the [./scripts/setup.sh](./scripts/setup.sh) file to see what is being installed on your machine.

## 📂 Project Structure

Here's an overview of the project's directory structure:

- `crates/`: Contains the main Rust crates for the project
    - `fuel-data-parser/`: Utility library for encoding/decoding data
    - `fuel-message-broker/`: Message broker implementation
    - `fuel-streams/`: Main fuel-streams package
    - `fuel-streams-core/`: Core components for working with streams
    - `fuel-streams-domains/`: Domain-specific implementations
    - `fuel-streams-macros/`: Macro utilities for the project
    - `fuel-streams-store/`: Storage implementations
    - `fuel-streams-types/`: Common types and traits
    - `fuel-streams-test/`: Testing utilities
    - `sv-publisher/`: Publisher service implementation
    - `sv-consumer/`: Consumer service implementation
    - `sv-webserver/`: WebSocket server implementation
- `benches/`: Benchmarking code
- `tests/`: Integration and end-to-end tests
- `examples/`: Example code and usage demonstrations
- `cluster/`: Kubernetes cluster configuration and deployment files
- `scripts/`: Utility scripts for setup, deployment, and maintenance

## 🧪 Running Tests

The project uses cargo-nextest for running tests. Here are the available test commands:

```sh
# Run all tests in the project
make test

# Run tests for a specific package
make test PACKAGE=<package-name>

# Run tests in watch mode
make test-watch

# Run tests with a specific profile
make test PROFILE=<profile-name>

# Run Helm chart tests
make helm-test
```

## 🔍 Development Commands

### Building Commands

| Command          | Description                              |
| ---------------- | ---------------------------------------- |
| `make build`     | Build release version                    |
| `make dev-watch` | Run in development mode with auto-reload |

### Formatting Commands

| Command             | Description                      |
| ------------------- | -------------------------------- |
| `make fmt`          | Format all code                  |
| `make fmt-rust`     | Format Rust code only            |
| `make fmt-markdown` | Format markdown files            |
| `make fmt-prettier` | Format other files with prettier |

### Linting Commands

| Command              | Description                   |
| -------------------- | ----------------------------- |
| `make lint`          | Run all linters               |
| `make lint-rust`     | Run Rust linter               |
| `make lint-clippy`   | Run clippy                    |
| `make lint-markdown` | Lint markdown files           |
| `make lint-machete`  | Check for unused dependencies |

### Docker Commands

| Command             | Description           |
| ------------------- | --------------------- |
| `make start-docker` | Start Docker services |
| `make stop-docker`  | Stop Docker services  |
| `make reset-docker` | Reset Docker services |

### Database Commands

| Command         | Description     |
| --------------- | --------------- |
| `make setup-db` | Set up database |
| `make reset-db` | Reset database  |

### Documentation Commands

| Command           | Description                 |
| ----------------- | --------------------------- |
| `make docs`       | Generate documentation      |
| `make docs-serve` | Serve documentation locally |

### Version Management Commands

| Command                           | Description                      |
| --------------------------------- | -------------------------------- |
| `make version`                    | Show current version             |
| `make bump-version VERSION=X.Y.Z` | Bump version to specified number |

### Audit Commands

| Command               | Description          |
| --------------------- | -------------------- |
| `make audit`          | Run security audit   |
| `make audit-fix-test` | Test security fixes  |
| `make audit-fix`      | Apply security fixes |

### Cluster Commands

| Command                | Description             |
| ---------------------- | ----------------------- |
| `make minikube-setup`  | Setup Minikube cluster  |
| `make minikube-start`  | Start Minikube cluster  |
| `make minikube-delete` | Delete Minikube cluster |
| `make cluster-up`      | Start local cluster     |
| `make cluster-down`    | Stop local cluster      |
| `make cluster-reset`   | Reset local cluster     |

### Load Testing Commands

| Command          | Description    |
| ---------------- | -------------- |
| `make load-test` | Run load tests |
| `make bench`     | Run benchmarks |

## 🚀 Running Local Services

The project includes several services that can be run locally:

### Publisher Service

The publisher service is responsible for fetching and publishing blockchain data. You can run it in different modes:

```sh
# Run publisher in development mode
make run-publisher-testnet-dev # For testnet
make run-publisher-mainnet-dev # For mainnet

# Run with profiling
make run-publisher-testnet-profiling
make run-publisher-mainnet-profiling

# Run with custom parameters
make run-publisher \
    NETWORK=testnet \
    MODE=dev \
    PORT=4000 \
    TELEMETRY_PORT=9001 \
    NATS_URL=localhost:4222 \
    FROM_HEIGHT=0
```

- Use `testnet-dev` when developing features against testnet
- Use `mainnet-dev` when developing features against mainnet
- Use `*-profiling` modes when you need to analyze performance or debug memory issues
- Custom parameters:
    - `NETWORK`: Choose between `testnet` or `mainnet`
    - `MODE`: Choose between `dev` or `profiling`
    - `PORT`: Service port (default: 4000)
    - `TELEMETRY_PORT`: Metrics port (default: 9001)
    - `NATS_URL`: NATS server URL
    - `FROM_HEIGHT`: Starting block height

### Consumer Service

The consumer service processes the published blockchain data and maintains the database state:

```sh
# Run with default settings
make run-consumer

# Run with custom parameters
make run-consumer \
    NATS_URL=localhost:4222 \
    PORT=9002
```

- Custom parameters:
    - `NATS_URL`: NATS server URL (default: localhost:4222)
    - `PORT`: Service port (default: 9002)

This service should be running alongside the publisher to process the data stream.

### Webserver

The webserver provides WebSocket endpoints for clients to subscribe to real-time blockchain data:

```sh
# Run with default settings
make run-webserver-testnet-dev
make run-webserver-mainnet-dev

# Run with custom parameters
make run-webserver \
    NETWORK=testnet \
    MODE=dev \
    PORT=9003 \
    NATS_URL=nats://localhost:4222
```

- Use `testnet-dev` to serve testnet data during development
- Use `mainnet-dev` to serve mainnet data during development
- Custom parameters:
    - `NETWORK`: Choose between `testnet` or `mainnet`
    - `MODE`: Choose between `dev` or `profiling`
    - `PORT`: Service port (default: 9003)
    - `NATS_URL`: NATS server URL

For local development, a typical setup would be:

1. Start the publisher service for your desired network
2. Run the consumer service to process the data
3. Start the webserver to expose the processed data via WebSocket

## 📇 Code conventions

We enforce some conventions to ensure code quality, sustainability, and maintainability. The following tools help us with that:

- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) - Ensures that commit messages are clear and understandable.
- [Pre-commit](https://pre-commit.com/) - Ensures that the code is formatted and linted before being committed.
- [Commitlint](https://commitlint.js.org/) - Lints commit messages to ensure they follow the Conventional Commits specification.

### 📝 Writing your Commits & Pull Requests

When creating a commit, please follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification. Use `category(scope or module): message` in your commit message with one of the following categories:

- `build`: Changes regarding the build of the software, dependencies, or the addition of new dependencies.
- `ci`: Changes regarding the configuration of continuous integration (e.g., GitHub Actions, CI systems).
- `docs`: Changes to existing documentation or creation of new documentation (e.g., README, usage docs).
- `feat`: All changes that introduce completely new code or new features.
- `fix`: Changes that fix a bug (ideally referencing an issue if present).
- `perf`: Changes that improve the performance of the software.
- `refactor`: Any code-related change that is not a fix or a feature.
- `test`: Changes regarding tests (adding new tests or changing existing ones).

This is a general rule used for commits. When you are creating a PR, ensure that the title follows the same pattern, but in terms of PR, the scope is a mandatory field. That's the scopes allowed at the moment:

- `repo`: Changes that affect a global scope of the repository.
- `release`: Scoped used for automatic release pull requests.
- `core`: Changes that affect the core package.
- `publisher`: Changes that affect the publisher package.
- `fuel-streams`: Changes that affect the fuel-streams package.
- `benches`: Changes related to benchmarks.
- `deps`: Changes related to dependencies.
- `macros`: Changes that affect the macros package.

## 🚀 Running Local Cluster

The project includes support for running a local Kubernetes cluster using [Minikube](https://minikube.sigs.k8s.io/docs/start/) for development and testing. Here's a quick guide to get started:

1. Setup Minikube cluster:

```sh
make minikube-setup
make minikube-start
```

For detailed information about the necessary tools to install, cluster configuration, deployment options, and troubleshooting, please refer to the [Cluster Documentation](./cluster/README.md).

## 📬 Open a Pull Request

1. Fork this repository and clone your fork.
2. Create a new branch out of the `main` branch with the naming convention `<username>/<category>/<branch-name>`.
3. Make and commit your changes following the conventions described above.
4. Ensure the title of your PR is clear, concise, and follows the pattern `<category(scope): message>`.
5. Ensure pre-commit checks pass by running `make lint`.
6. Push your changes and open a pull request.

## 🛠 Troubleshooting

If you encounter any issues while setting up or contributing to the project, here are some common problems and their solutions:

1. **Pre-commit hooks failing**: Ensure you've installed all the required dependencies and run `make setup`. If issues persist, try running `pre-commit run --all-files` to see detailed error messages.

2. **Build failures**: Make sure you're using the latest stable Rust version (1.84.0) and the correct nightly version (nightly-2025-01-24). You can update Rust using `rustup update stable` and `rustup update nightly-2025-01-24`.

3. **Test failures**: If specific tests are failing, try running them in isolation to see if it's a concurrency issue. Use `RUST_BACKTRACE=1` to get more detailed error information.

4. **Docker issues**: If you encounter Docker-related issues, try:

    - Ensuring Docker daemon is running
    - Running `docker system prune` to clean up unused resources
    - Checking Docker logs with `docker logs <container-name>`

5. **Database issues**: If you encounter database problems:
    - Ensure PostgreSQL is running with `make start-docker`
    - Reset the database with `make reset-db`
    - Check database logs with `docker logs <postgres-container-name>`

If you encounter any other issues not listed here, please open an issue on the GitHub repository.

## 📚 Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Fuel Labs Documentation](https://docs.fuel.network/)
- [NATS Documentation](https://docs.nats.io/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Tilt Documentation](https://docs.tilt.dev/)

We appreciate your contributions to the Fuel Data Systems project!
