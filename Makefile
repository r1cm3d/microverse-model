.PHONY: all build run test clean fmt clippy doc scraper preprocess train help

# Default target
all: build

# Build the project
build:
	cargo build

# Build for release
release:
	cargo build --release

# Run the project
run:
	cargo run

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt

# Check code with clippy
clippy:
	cargo clippy -- -D warnings

# Build documentation
doc:
	cargo doc --no-deps --open

# Clean build artifacts
clean:
	cargo clean

# Run the example
run-example:
	cargo run --example basic_usage

# Run the scraper
scraper:
	cargo run --bin scraper

# Run the preprocessor
preprocess:
	cargo run --bin preprocessor

# Run the training loop
train:
	cargo run --release -- train

# Help target
help:
	@echo "Available targets:"
	@echo "  build       - Build the project (dev)"
	@echo "  release     - Build the project (release)"
	@echo "  run         - Run the project"
	@echo "  test        - Run tests"
	@echo "  fmt         - Format code using rustfmt"
	@echo "  clippy      - Run clippy linter"
	@echo "  doc         - Build and open documentation"
	@echo "  clean       - Clean build artifacts"
	@echo "  run-example - Run the basic_usage example"
	@echo "  scraper     - Run the transcript scraper"
	@echo "  preprocess  - Run the data preprocessor"
	@echo "  train       - Run the training loop (release build)"
	@echo "  help        - Show this help message"
