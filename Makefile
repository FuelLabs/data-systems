setup:
	./scripts/install.sh

check:
	cargo check --all-targets --all-features

build:
	cargo build --release

dev:
	cargo run

dev-watch:
	cargo watch -- cargo run

install:
	cargo fetch

lint:
	cargo fmt -- --check --color always
	cargo clippy --all-targets --all-features -- -D warnings
	pre-commit run --all-files

run:
	cargo run --release
