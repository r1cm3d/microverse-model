# Active Context

## Current Focus
Transitioning from data collection to model architecture design and implementation. The scraping pipeline is complete, and the training dataset is available.

## Recent Changes
- **Memory Bank**: Updated all context files to reflect the project's actual goal (SLM for Rick and Morty) instead of the initial generic simulation brief.
- **Data Collection**: Implemented `scripts/scraper.rs` to fetch transcripts from the Rick and Morty Wiki.
- **Dataset**: Generated `datasets/rick_morty_all_transcripts.csv` containing seasons 1-8 dialogue.

## Active Decisions
- **Project Scope**: Confirmed focus on Small Language Model (SLM) trained on specific character data.
- **Data Source**: Using Rick and Morty Fandom Wiki as the primary source for transcripts.
- **Scraper Implementation**: Built as a standalone binary (`scripts/scraper.rs`) within the project structure to keep the core library clean.

## Next Steps
1.  **Data Analysis**: Analyze the scraped dataset for quality, balance (character line counts), and format suitable for tokenization.
2.  **Model Selection**: Research and select an appropriate Rust-compatible ML framework (e.g., Candle, Tch-rs).
3.  **Tokenizer**: Implement or integrate a tokenizer for the dataset.
4.  **Training Pipeline**: Begin setting up the training loop in `src/lib.rs`.
