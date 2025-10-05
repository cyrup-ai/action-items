# List available commands
default:
    @just --list

# Install development dependencies
setup:
    cargo install cargo-watch
    cargo install cargo-nextest

# Run the project in development mode with auto-reload
dev:
    cargo watch -x 'run -p action_items'

# Run tests in watch mode
test-watch:
    cargo watch -x 'nextest run'

# Build the project
build:
    cargo build -p action_items

# Run tests
test:
    cargo nextest run

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Run the main launcher application
run:
    cargo run -p action_items

# Run in release mode
run-release:
    cargo run -p action_items --release

# Check the code
check:
    cargo check
