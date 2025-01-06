# Contributing

This guide will show you how to run this project locally if you want to test or contribute to it.

## 🛠 Prerequisites

Most projects under the umbrella of data systems are written in Rust, so we prefer using Rust tooling and community standards. Ensure you have the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- [Rust Nightly](https://rust-lang.github.io/rustup/concepts/channels.html) (version nightly-2024-11-06)
- [Make](https://www.gnu.org/software/make/)
- [Pre-commit](https://pre-commit.com/#install)
- [NodeJS](https://nodejs.org/en/download/)
- [PNPM](https://pnpm.io/installation)
- [NATS](https://nats.io/download/)
- [Tilt](https://docs.tilt.dev/install.html)
- [Minikube](https://minikube.sigs.k8s.io/docs/start/)
- [Kubernetes](https://kubernetes.io/)

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
- `tests/`: Integration and end-to-end tests
- `examples/`: Example code and usage demonstrations
- `cluster/`: Kubernetes cluster configuration and deployment files
- `scripts/`: Utility scripts for setup, deployment, and maintenance

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
- `deps`: Changes related to dependencies.
- `macros`: Changes that affect the macros package.

## 📜 Useful Commands

To make your life easier, here are some commands to run common tasks in this project:

| Command                  | Description                                           |
| ------------------------ | ----------------------------------------------------- |
| `make install`           | Fetch the project dependencies using `cargo fetch`    |
| `make setup`             | Run the setup script located at `./scripts/setup.sh`  |
| `make create-env`        | Create environment configuration file                 |
| `make fmt`               | Format the code and Markdown files                    |
| `make lint`              | Perform linting checks on the code and Markdown files |
| `make test`              | Run all tests in the project                          |
| `make test-watch`        | Run tests in watch mode                               |
| `make clean`             | Clean the build artifacts                             |
| `make dev-watch`         | Run the project in development mode with auto-reload  |
| `make audit`             | Run security audit on dependencies                    |
| `make audit-fix`         | Fix security vulnerabilities in dependencies          |
| `make version`           | Show current version                                  |
| `make bump-version`      | Bump project version                                  |
| `make load-test`         | Run load tests                                        |
| `make run-publisher`     | Run the publisher with custom configuration           |
| `make run-mainnet-dev`   | Run publisher in mainnet dev mode                     |
| `make run-testnet-dev`   | Run publisher in testnet dev mode                     |
| `make validate-env`      | Validate environment setup                            |
| `make cleanup-artifacts` | Clean up old artifacts on Github                      |

## 🚀 Running Local Cluster

The project includes support for running a local Kubernetes cluster using [Minikube](https://minikube.sigs.k8s.io/docs/start/) for development and testing. Here's a quick guide to get started:

1. Setup Minikube cluster:

```sh
make minikube-setup
make minikube-start
```

For detailed information about the necessary tools to install, cluster configuration, deployment options, and troubleshooting, please refer to the [Cluster Documentation](./cluster/README.md).

## 🧪 Running Tests

To run all tests in the project, use:

```sh
make test
```

For running specific tests or test modules, you can use:

```sh
make test PACKAGE=<package-name>
```

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

2. **Build failures**: Make sure you're using the latest stable Rust version and the correct nightly version. You can update Rust using `rustup update stable` and `rustup update nightly-2024-11-06`.

3. **Test failures**: If specific tests are failing, try running them in isolation to see if it's a concurrency issue. Use `RUST_BACKTRACE=1` to get more detailed error information.

If you encounter any other issues not listed here, please open an issue on the GitHub repository.

## 📚 Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Fuel Labs Documentation](https://docs.fuel.network/)

We appreciate your contributions to the Fuel Data Systems project!
