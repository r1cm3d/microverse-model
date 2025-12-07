# Project Brief: Microverse Model

## Overview
Microverse Model is a Small Language Model (SLM) implemented in Rust, specifically designed to be trained on the complete collection of subtitles and transcripts from the **Rick and Morty** series. The project aims to explore the capabilities of SLMs when trained on highly specific, character-driven datasets.

## Key Features
- **Specialized Training Data**: Utilizes a comprehensive dataset of Rick and Morty transcripts scraped from fandom wikis.
- **Rust Implementation**: Leverages Rust's performance and memory safety for efficient model training and inference.
- **Data Collection Pipeline**: Includes a robust scraper to automate the retrieval and formatting of training data.
- **Extensible Architecture**: Designed to allow experimentation with different model architectures and training parameters.

## Core Requirements
- Rust 1.70 or higher.
- Access to the internet for data scraping (initially).
- `datasets/` directory for storing training data (CSV/JSON).
