# Contributing

This guide will show you how to run this project locally if you want to test
or contribute to it.

## 🛠 Prerequisites

Most projects under the umbrella of data systems are written in Rust, so we
prefer using Rust tooling and community standards. Ensure you have the
following tools installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Make](https://www.gnu.org/software/make/)
- [Pre-commit](https://pre-commit.com/#install)

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

## 📝 Code conventions

We enforce some conventions to ensure code quality, sustainability, and
maintainability. The following tools help us with that:

- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) -
  Ensures that commit messages are clear and understandable.
- [Pre-commit](https://pre-commit.com/) - Ensures that the code is formatted
  and linted before being committed.
- [Commitizen](https://commitizen-tools.github.io/commitizen/) - Standardizes
  commit messages using the Conventional Commits specification.

### Writing your commits

When creating a commit, please follow the [Conventional
Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification. Use
`category(scope or module): message` in your commit message with one of the
following categories:

- `feat / feature`: All changes that introduce completely new code or new
  features.
- `fix`: Changes that fix a bug (ideally referencing an issue if present).
- `refactor`: Any code-related change that is not a fix or a feature.
- `docs`: Changes to existing documentation or creation of new documentation
  (e.g., README, usage docs).
- `build`: Changes regarding the build of the software, dependencies, or the
  addition of new dependencies.
- `test`: Changes regarding tests (adding new tests or changing existing
  ones).
- `ci`: Changes regarding the configuration of continuous integration (e.g.,
  GitHub Actions, CI systems).
- `chore`: Changes to the repository that do not fit into any of the above
  categories.

### Using Commitizen

Commitizen helps you create commit messages that follow the Conventional
Commits specification. To use Commitizen, refer to the [Commitizen installation
guide](https://commitizen-tools.github.io/commitizen/).

Once installed, create your commit using the following command:

```sh
cz commit
```

Commitizen will guide you through the process of creating a standardized
commit message.

## 📜 Useful Commands

To make your life easier, here are some commands to run common tasks in this
project:

| Command          | Description                                            |
| ---------------- | ------------------------------------------------------ |
| `make build`     | Build the project                                      |
| `make check`     | Run cargo check                                        |
| `make dev-watch` | Run the project in development mode with auto-reload   |
| `make dev`       | Run the project in development mode                    |
| `make fmt`       | Format the code                                        |
| `make install`   | Install the project                                    |
| `make lint`      | Format and lint the code                               |
| `make run`       | Run the project in release mode                        |
| `make setup`     | Install all the tools needed                           |

## 📬 Open a Pull Request

1. Fork this repository and clone your fork.
2. Create a new branch out of the `master` branch with the naming convention
   `<username>/<fix|feat|chore|build|docs>/<branch-name>`.
3. Make and commit your changes following the conventions described above.
4. Ensure the title of your PR is clear, concise, and follows the pattern
   `<category(scope): message>`.
5. Ensure pre-commit checks pass by running `make lint`.
