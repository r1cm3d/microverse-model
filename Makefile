.PHONY: all build run test clean fmt clippy doc scraper preprocess train generate speak venv help

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

CHECKPOINT ?=
CHARACTER  ?= RICK
INPUT      ?=
OUTPUT     ?= output.wav
PYTHON     ?= .venv/bin/python3

# Create Python virtual environment and install TTS dependencies
# Requires uv (sudo pacman -S uv); uv downloads Python 3.11 automatically
venv:
	uv venv --python 3.11 .venv
	uv pip install --python .venv/bin/python3 -r python/requirements.txt

# Run text generation (CHECKPOINT=path required)
generate:
	cargo run --release -- generate \
		--checkpoint $(CHECKPOINT) \
		--character $(CHARACTER) \
		$(if $(INPUT),--input "$(INPUT)",)

# Run voice synthesis (CHECKPOINT=path required); uses .venv by default
speak:
	cargo run --release -- speak \
		--checkpoint $(CHECKPOINT) \
		--character $(CHARACTER) \
		--output $(OUTPUT) \
		--python $(PYTHON) \
		$(if $(INPUT),--input "$(INPUT)",)

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
	@echo "  generate    - Run text generation (CHECKPOINT=path required)"
	@echo "  speak       - Run voice synthesis (CHECKPOINT=path required)"
	@echo "  venv        - Create .venv and install python/requirements.txt"
	@echo "  help        - Show this help message"
