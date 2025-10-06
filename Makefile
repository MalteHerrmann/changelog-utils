# Run tests
.PHONY: test test-ci
test:
	cargo nextest run

test-ci:
	cargo nextest run --verbose --profile CI --features remote

# Build binary locally
build:
	cargo build --locked

# Format codebase
format:
	cargo fmt

# Install binary
install:
	cargo install --path . --locked

# Run Docker image
docker-run:
	docker run -v $(PWD):/data MalteHerrmann/changelog-utils lint

# Build docker image
docker-build:
	docker build -t MalteHerrmann/changelog-utils .

# Lint codebase
lint:
	cargo clippy

PHONY: build format install docker-run docker-build lint
