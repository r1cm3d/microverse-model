# System Patterns

## Architecture

The project consists of two main components:
1.  **Data Ingestion (Scraper)**: A standalone binary responsible for fetching, parsing, and cleaning transcript data.
2.  **Model Engine (Library/Binary)**: The core SLM implementation (currently in early stages).

### Directory Structure
```
microverse-model/
├── src/
│   ├── lib.rs          # Core model logic (placeholder)
│   └── main.rs         # Inference/Training CLI (placeholder)
├── scripts/
│   └── scraper.rs      # Data collection tool
├── datasets/
│   └── rick_morty_all_transcripts.csv  # Training data
├── examples/           # Usage examples
└── docs/memorybank/    # Project documentation & context
```

## Design Patterns

- **Scripts as Binaries**: Auxiliary tools like the scraper are located in `scripts/` but defined as `[[bin]]` targets in `Cargo.toml`, allowing them to be run via `cargo run --bin scraper`.
- **Data-Driven**: The model relies heavily on the quality and structure of the `datasets/` directory. The scraper ensures data is normalized (CSV/JSON).
- **Asynchronous I/O**: The scraper uses `tokio` and `reqwest` for efficient, non-blocking network requests to fetch episode data.

## Data Pipeline
1.  **Fetch**: `scraper` retrieves episode lists from the wiki.
2.  **Parse**: HTML is parsed using `scraper` crate to extract dialogue lines.
3.  **Clean**: Text is normalized (whitespace trimming, quote handling).
4.  **Store**: Data is saved to `datasets/` in both JSON (rich structure) and CSV (flat structure) formats.
