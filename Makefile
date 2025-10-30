# Makefile for Q9gent

.PHONY: help build run test clean fmt clippy check install release docker

# Default target
help:
	@echo "Q9gent - Lightweight Rust CLI Assistant Server"
	@echo ""
	@echo "Available targets:"
	@echo "  build      - Build debug binary"
	@echo "  release    - Build optimized release binary"
	@echo "  run        - Run the server (debug mode)"
	@echo "  test       - Run all tests"
	@echo "  check      - Check compilation without building"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run clippy linter"
	@echo "  clean      - Remove build artifacts"
	@echo "  install    - Install to ~/.cargo/bin"
	@echo "  docker     - Build Docker image"
	@echo ""

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Run the server
run:
	RUST_LOG=q9gent=debug cargo run

# Run tests
test:
	cargo test

# Check compilation
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy
clippy:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf sessions/

# Install to system
install:
	cargo install --path .

# Build Docker image
docker:
	docker build -t q9gent:latest .

# Cross-compile for Windows
windows:
	cargo build --release --target x86_64-pc-windows-gnu

# Cross-compile for Linux (musl)
linux-musl:
	cargo build --release --target x86_64-unknown-linux-musl

# Run all quality checks
ci: fmt clippy test
	@echo "✓ All CI checks passed"
