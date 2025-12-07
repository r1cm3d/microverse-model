# Microverse Model

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A SLM implemented in Rust, using all subtitles from the Rick and Morty series as training data. 

## 🔧 Development

### Prerequisites

- Rust 1.70 or higher
- Cargo (included with Rust)

### Using Makefile

The project includes a `Makefile` for common development tasks.

Run `make help` to see available targets.

```bash
make help
```

Common commands:

- `make build`: Build the project (dev)
- `make release`: Build the project (release)
- `make run`: Run the project
- `make test`: Run tests
- `make fmt`: Format code using rustfmt
- `make clippy`: Run clippy linter
- `make doc`: Build and open documentation
- `make clean`: Clean build artifacts
- `make run-example`: Run the basic_usage example
- `make scraper`: Run the transcript scraper

If you prefer using `cargo` directly:

- Build: `cargo build --release`
- Test: `cargo test`
- Bench: `cargo bench`
- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`

## 📄 License

This project is licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)
