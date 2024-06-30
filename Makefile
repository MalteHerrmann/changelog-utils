# Run tests
test:
	cargo test

# Build binary locally
build:
	cargo build . --locked

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
