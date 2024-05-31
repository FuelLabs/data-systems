# Contributing

This guide will show you how to run this project locally if you want to test or
contribute to this project.

## 🛠 Prerequisites

Most projects here under the umbrella of data systems are written in Rust, so
we have preference for using Rust tooling and standards from the community as
much as possible. So, this are the base tooling you need to ensure have
installed on to run this project.

- [Rust](https://www.rust-lang.org/tools/install)
- [Make](https://www.gnu.org/software/make/)
- [Pre-commit](https://pre-commit.com/#install)

## 📟 Setting up

First, you need to clone this repository:

```sh
git clone git@github.com:FuelLabs/data-systems.git
cd data-systems
```

Now you need to install few tools to ensure the code quality and standards are
met. We simplify this process for you using Make, so since you have it
installed in your machine, you can simply run:

```sh
make setup
```

You can check the [./scripts/setup.sh](./scripts/setup.sh) file to see what is
being installed on your machine.

## 📝 Code conventions

Some conventions are enforced here not to just ensure code quality, but also
helps the project to be more sustainable and maintainable. So, we have some
tools to help us with that.

- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) - We
  use this standard to ensure that the commit messages are clear and
  understandable.
- [Pre-commit](https://pre-commit.com/) - We use this tool to ensure that the
  code is formatted and linted before being committed.

### Writing your commits

When you create a commit we kindly ask you to follow the [Conventional
Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.
Use `category(scope or module): message` in your commit message while using one of
the following categories:

- `feat / feature`: all changes that introduce completely new code or new
  features
- `fix`: changes that fix a bug (ideally you will additionally reference an
  issue if present)
- `refactor`: any code related change that is not a fix nor a feature
- `docs`: changing existing or creating new documentation (i.e. README, docs for
  usage of a lib or cli usage)
- `build`: all changes regarding the build of the software, changes to
  dependencies or the addition of new dependencies
- `test`: all changes regarding tests (adding new tests or changing existing
  ones)
- `ci`: all changes regarding the configuration of continuous integration (i.e.
  github actions, ci system)
- `chore`: all changes to the repository that do not fit into any of the above
  categories

## 📜 Useful Commands

To make your life easier, we have some commands that you can use to run the most
common tasks on this project.

| Command          | Description                                            |
| ---------------- | ------------------------------------------------------ |
| `make build`     | Build the project                                      |
| `make check`     | Run cargo check                                        |
| `make dev-watch` | Run the project in a development mode with auto-reload |
| `make dev`       | Run the project in a development mode                  |
| `make lint`      | Format and lint the code                               |
| `make run`       | Run the project in a release mode                      |
| `make setup`     | Install all the tools needed                           |

## 📬 Open a Pull Request

1. Fork this repository and clone your fork
2. Create a new branch out of the `master` branch with the naming convention `<username>/<fix|feat|chore|build|docs>/<branch-name>`.
3. Make and commit your changes following the conventions described above.
4. Ensure the title of your PR is clear, concise, and follows the pattern `<category(scope): message>`.
5. Ensure pre-commit checks pass by running `make lint`.
