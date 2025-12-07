# Tech Context

## Technology Stack
- **Language**: Rust (Edition 2021, Version 1.70+)
- **Build System**: Cargo
- **Package Registry**: crates.io

## Development Environment
- **IDE**: RustRover
- **Operating System**: Linux 6.17

## Tooling
- **Formatting**: `rustfmt`
- **Linting**: `cargo clippy`
- **Testing**: Native `cargo test` framework
- **Documentation**: `cargo doc`

## Dependencies
### Core/Model (Planned)
- (TBD - Likely `tch-rs` (PyTorch) or `candle` (Hugging Face) for ML capabilities)

### Scraper / Data Processing
- **reqwest**: HTTP client for fetching web pages.
- **scraper**: HTML parsing and element selection.
- **serde**: Serialization framework for handling JSON/CSV data.
- **serde_json**: JSON support.
- **tokio**: Async runtime for efficient scraping.

## CI/CD
- **Platform**: GitHub Actions (implied).
- **Workflows**: Build, Test, Format, Clippy.
