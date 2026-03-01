# Microverse Model

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust: 1.75+](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A Small Language Model (SLM) built entirely in Rust and trained on Rick and Morty episode
transcripts from Seasons 1–8. The goal is to capture character-specific speech patterns and
enable style transfer — generating new dialogue in the voice of any character from the show.

---

## Features

- Fandom wiki scraper — collects 8 seasons, 8,323 dialogue lines
- Data cleaning and train/val split preprocessor
- Byte-level tokenizer (vocab size 256)
- GPT-style decoder transformer (4 layers, 128 d\_model, 4 heads, 256 context)
- AdamW training loop with safetensors checkpointing and resume support
- Autoregressive inference with temperature, top-k, and top-p sampling
- Character-conditioned style transfer via prompt prefix

---

## Milestones

| # | Title | Status | Acceptance Criteria |
|---|-------|--------|---------------------|
| 1 | Data Ready | done | Transcripts collected and cleaned |
| 2 | Model Trains Successfully | done | Loss decreases; checkpoints save/load |
| 3 | Quality Text Generation | done | Coherent dialogue; character voices distinct; style transfer works |
| 4 | Voice Synthesis Works | pending | Text-to-audio; recognizable Rick/Morty voices |
| 5 | End-to-End Demo | pending | Full pipeline in one command; demo video |

Full milestone history: https://github.com/r1cm3d/microverse-model/milestones

---

## Architecture

```
scripts/scraper.rs        -- web scraper binary (fandom wiki -> CSV)
scripts/preprocessor.rs   -- data cleaning binary (CSV -> train/val corpus)
src/
  lib.rs                  -- crate root; exports tokenizer, dataset, model, train, generate
  tokenizer.rs            -- byte-level tokenizer
  dataset.rs              -- corpus loader and random batch sampler
  model.rs                -- GPT decoder (CausalSelfAttention, FeedForward, TransformerBlock)
  train.rs                -- training loop (AdamW, checkpointing)
  generate.rs             -- autoregressive sampling (temperature, top-k, top-p)
  main.rs                 -- CLI entry point (train / generate subcommands)
```

### Model defaults

| Parameter | Value |
|-----------|-------|
| vocab\_size | 256 |
| d\_model | 128 |
| n\_heads | 4 |
| n\_layers | 4 |
| context\_len | 256 |
| d\_ff | 512 |

Checkpoints are stored in safetensors format under `checkpoints/`.

---

## Dataset

- Source: rickandmorty.fandom.com, Seasons 1–8
- 8,323 raw dialogue lines after scraping and cleaning
- Split: 7,490 train lines / 833 validation lines
- Output files: `datasets/train_corpus.txt`, `datasets/val_corpus.txt`

The processed dataset files are committed to the repository — scraping and preprocessing are
optional steps needed only to regenerate them.

---

## Prerequisites

- Rust 1.75 or higher with Cargo

---

## Getting Started

```bash
git clone https://github.com/r1cm3d/microverse-model.git
cd microverse-model
make build           # dev build
make test            # run all tests
make train           # start training (datasets already committed)
make generate CHECKPOINT=checkpoints/ckpt_000100.safetensors
```

To regenerate the dataset from scratch (optional):

```bash
make scraper         # re-scrape fandom wiki -> datasets/rick_morty_all_transcripts.csv
make preprocess      # clean CSV -> train_corpus.txt + val_corpus.txt
```

---

## CLI Reference

### train

```
cargo run --release -- train [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--train-data` | `datasets/train_corpus.txt` | Training corpus path |
| `--val-data` | `datasets/val_corpus.txt` | Validation corpus path |
| `--checkpoint-dir` | `checkpoints` | Directory to save checkpoints |
| `--max-steps` | `5000` | Number of training steps |
| `--lr` | `3e-4` | Learning rate |
| `--batch-size` | `32` | Batch size |
| `--resume-from` | — | Path to checkpoint to resume from |

### generate

```
cargo run --release -- generate --checkpoint <PATH> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--checkpoint` | (required) | Path to a `.safetensors` checkpoint |
| `--character` | `RICK` | Character voice prefix |
| `--input` | — | Text to stylize |
| `--max-tokens` | `200` | Maximum tokens to generate |
| `--temperature` | `0.8` | Sampling temperature |
| `--top-k` | `40` | Top-k cutoff (0 = disabled) |
| `--top-p` | `0.9` | Nucleus sampling threshold |
| `--seed` | `42` | Random seed |

### Make targets

| Target | Description |
|--------|-------------|
| `build` | Dev build |
| `release` | Release build |
| `run` | Run main binary |
| `test` | Run all tests |
| `fmt` | Format code with rustfmt |
| `clippy` | Run clippy linter |
| `doc` | Build and open documentation |
| `clean` | Remove build artifacts |
| `run-example` | Run the basic\_usage example |
| `scraper` | Re-scrape fandom wiki |
| `preprocess` | Run data preprocessor |
| `train` | Start training loop (release build) |
| `generate` | Run text generation (`CHECKPOINT=path` required) |
| `help` | Print all targets |

---

## Contributing

Contributions are welcome. To submit a change:

1. Fork the repository and create a feature branch
2. Make your changes
3. Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` — all must pass
4. Open a pull request with a clear description of what changed and why

---

## License

Licensed under MIT OR Apache-2.0.

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)
