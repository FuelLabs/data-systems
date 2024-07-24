# Contributing

This guide will show you how to run this project locally if you want to test
or contribute to it.

## 🛠 Prerequisites

Most projects under the umbrella of data systems are written in Rust, so we
prefer using Rust tooling and community standards. Ensure you have the
following tools installed:

-   [Rust](https://www.rust-lang.org/tools/install)
-   [Make](https://www.gnu.org/software/make/)
-   [Pre-commit](https://pre-commit.com/#install)
-   [NodeJS](https://nodejs.org/en/download/)

## 📟 Setting up

First, clone this repository:

```sh
git clone git@github.com:fuellabs/data-systems.git
cd data-systems
```

Now, install the necessary tools to ensure code quality and standards. Use
Make to simplify this process:

```sh
make setup
```

You can check the [./scripts/setup.sh](./scripts/setup.sh) file to see what is
being installed on your machine.

## 📇 Code conventions

We enforce some conventions to ensure code quality, sustainability, and
maintainability. The following tools help us with that:

-   [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) -
    Ensures that commit messages are clear and understandable.
-   [Pre-commit](https://pre-commit.com/) - Ensures that the code is formatted
    and linted before being committed.
-   [Commitlint](https://commitlint.js.org/) - Lints commit messages to ensure
    they follow the Conventional Commits specification.

### 📝 Writing your Commits & Pull Requests

When creating a commit, please follow the [Conventional
Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification. Use
`category(scope or module): message` in your commit message with one of the
following categories:

-   `build`: Changes regarding the build of the software, dependencies, or the
    addition of new dependencies.
-   `ci`: Changes regarding the configuration of continuous integration (e.g.,
    GitHub Actions, CI systems).
-   `docs`: Changes to existing documentation or creation of new documentation
    (e.g., README, usage docs).
-   `feat`: All changes that introduce completely new code or new
    features.
-   `fix`: Changes that fix a bug (ideally referencing an issue if present).
-   `perf`: Changes that improve the performance of the software.
-   `refactor`: Any code-related change that is not a fix or a feature.
-   `test`: Changes regarding tests (adding new tests or changing existing
    ones).

This is a general rule used for commits. When you are creating a PR, ensure
that the title follows the same pattern, but in terms of PR, the scope is a
mandatory field. That's the scopes allowed at the moment:

-   `repo`: Changes that affect a global scope of the repository.
-   `release`: Scoped used for automatic release pull requests.
-   `core`: Changes that affect the core package.
-   `publisher`: Changes that affect the publisher package.
-   `fuel-streams`: Changes that affect the data-streams package.

## 📜 Useful Commands

To make your life easier, here are some commands to run common tasks in this
project:

| Command          | Description                                           |
| ---------------- | ----------------------------------------------------- |
| `make build`     | Build the project with default settings               |
| `make clean`     | Clean the build artifacts and release directory       |
| `make dev-watch` | Run the project in development mode with auto-reload  |
| `make dev`       | Run the project in development mode                   |
| `make fmt`       | Format the code and Markdown files                    |
| `make install`   | Fetch the project dependencies using `cargo fetch`    |
| `make lint`      | Perform linting checks on the code and Markdown files |
| `make run`       | Run the built executable using `cargo run --release`  |
| `make setup`     | Run the setup script located at `./scripts/setup.sh`  |

## 📬 Open a Pull Request

1. Fork this repository and clone your fork.
2. Create a new branch out of the `main` branch with the naming convention
   `<username>/<category>/<branch-name>`.
3. Make and commit your changes following the conventions described above.
4. Ensure the title of your PR is clear, concise, and follows the pattern
   `<category(scope): message>`.
5. Ensure pre-commit checks pass by running `make lint`.
