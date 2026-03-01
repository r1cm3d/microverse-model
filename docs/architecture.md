# Architecture

## Overview

Microverse Model has three independent Rust binaries that cover distinct phases of the pipeline, plus a Python script for voice synthesis:

| Binary / Script | Path | Responsibility |
|-----------------|------|----------------|
| `scraper` | `scripts/scraper.rs` | Fetch and parse episode transcripts from fandom wiki |
| `preprocessor` | `scripts/preprocessor.rs` | Clean text, normalize unicode, split train/val corpus |
| `microverse-model` | `src/main.rs` | Train, generate text, and synthesize audio |
| `tts.py` | `python/tts.py` | XTTS-v2 voice synthesis (spawned as subprocess) |

Data flows one-way through the pipeline: scraper → preprocessor → training → generation → TTS.

## C4 Level 1 — System Context

> Source: [`diagrams/c4-context.puml`](diagrams/c4-context.puml)

The developer interacts with the CLI. The system reaches out to two external dependencies:
- **rickandmorty.fandom.com** — scraped once to build the dataset; not required at inference time.
- **Coqui XTTS-v2** — a pre-trained voice-cloning model invoked through a Python subprocess during the `speak` command.

Reference audio clips (used for voice cloning) are prepared manually by the developer using `yt-dlp` and `ffmpeg`, then placed in `audio_samples/`.

## C4 Level 2 — Containers

> Source: [`diagrams/c4-containers.puml`](diagrams/c4-containers.puml)

```
Developer
  │
  ├─ [scraper binary] ──HTTP──▶ rickandmorty.fandom.com
  │       └──writes──▶ datasets/rick_morty_all_transcripts.csv
  │
  ├─ [preprocessor binary]
  │       ├──reads──▶  datasets/rick_morty_all_transcripts.csv
  │       ├──writes──▶ datasets/rick_morty_transcripts_clean.csv
  │       └──writes──▶ datasets/train_corpus.txt + val_corpus.txt
  │
  └─ [microverse-model binary]
          ├──reads──▶  datasets/train_corpus.txt
          ├──writes──▶ checkpoints/ckpt_NNNNNN.safetensors
          └──spawn──▶  [python/tts.py]
                            └──reads──▶ audio_samples/*_reference.wav
                            └──writes──▶ output.wav
```

## C4 Level 3 — Components (microverse-model binary)

> Source: [`diagrams/c4-components.puml`](diagrams/c4-components.puml)

The main binary is split into focused modules with clear dependency direction:

```
main.rs (CLI routing)
  ├── train.rs      uses ──▶ model.rs, dataset.rs, checkpoints/
  ├── generate.rs   uses ──▶ model.rs, tokenizer.rs, checkpoints/
  └── tts_bridge.rs uses ──▶ python/tts.py (subprocess)

lib.rs — crate root
  exports: DialogueLine, write_dialogues_to_csv
  declares: pub mod dataset, generate, model, tokenizer, train, tts_bridge
```

No module imports from `main.rs`. No circular dependencies. `tts_bridge.rs` depends only on `std::process::Command` — it is intentionally decoupled from the ML stack.

## Data Flow

> Source: [`diagrams/data-flow.puml`](diagrams/data-flow.puml)

The full transformation pipeline from raw web content to synthesized audio:

```
rickandmorty.fandom.com
  │  (HTTPS/HTML)
  ▼
scraper  →  rick_morty_all_transcripts.csv  (~8,360 rows)
  │
  ▼
preprocessor  →  rick_morty_transcripts_clean.csv  (~8,323 rows)
              →  train_corpus.txt  (7,490 lines)
              →  val_corpus.txt      (833 lines)
  │
  ▼
tokenizer.encode()  →  Vec<u32>  (byte-level, vocab=256)
  │
  ▼
GPT forward pass  →  logits (B, T, 256)
  │
  ▼
sampler  →  generated text  (autoregressive, top-k + top-p)
  │
  ▼
tts_bridge + tts.py  →  output.wav  (XTTS-v2 voice cloning)
```

## Key Design Decisions

**Byte-level tokenization.** Vocabulary of 256 (one per byte value). No special tokens, no pre-tokenization step, handles any valid UTF-8 including character names. Simpler than BPE at the cost of longer sequences.

**Pre-LayerNorm.** Each TransformerBlock applies LayerNorm before the attention and feed-forward sublayers, not after. This produces more stable gradients during training compared to the original post-norm GPT architecture.

**CPU-first.** No GPU requirement. The model is intentionally small (d_model=128, 4 layers, ~2.6M parameters) so training and inference are feasible on a laptop CPU.

**Safetensors checkpoints.** The Hugging Face safetensors format is used for all checkpoints. It is efficient, safe (no arbitrary code execution), and interoperable with Python tooling.

**Python bridge for TTS.** XTTS-v2 has no stable Rust binding. Rather than writing FFI or a custom binding, `tts_bridge.rs` spawns `python/tts.py` as a subprocess. This keeps the Rust codebase free of Python dependencies while remaining straightforward to replace if a Rust TTS library becomes available.

**Separate binaries for data tools.** The scraper and preprocessor are declared as `[[bin]]` targets in `Cargo.toml`, not as library functions. This keeps data-collection concerns cleanly separated from the model library and avoids pulling HTTP/HTML parsing dependencies into the main binary's dependency graph.
