# Microverse Model — Documentation

Microverse Model is a Small Language Model (SLM) built in Rust and trained on Rick and Morty episode transcripts (Seasons 1–8). It generates character-conditioned dialogue and synthesizes audio using XTTS-v2 voice cloning.

## Quick Start

```bash
# 1. Build
make build

# 2. (Optional) Re-scrape transcripts
make scraper

# 3. Preprocess data
make preprocess

# 4. Train
make train

# 5. Generate text
make generate CHECKPOINT=checkpoints/ckpt_005000.safetensors CHARACTER=RICK

# 6. Synthesize audio (requires reference WAVs — see voice-synthesis.md)
make venv
make speak CHECKPOINT=checkpoints/ckpt_005000.safetensors CHARACTER=RICK OUTPUT=rick.wav
```

## Contents

| Document | Description |
|----------|-------------|
| [architecture.md](architecture.md) | System context, container map, and module dependency overview |
| [model.md](model.md) | GPT decoder architecture, layer configuration, and forward pass |
| [training.md](training.md) | Training loop, optimizer, checkpointing, and hyperparameters |
| [generation.md](generation.md) | Autoregressive decoding and sampling strategies |
| [voice-synthesis.md](voice-synthesis.md) | XTTS-v2 integration, TTS bridge, and Python subprocess |
| [cli.md](cli.md) | All CLI subcommands, flags, and Makefile targets |
| [reference-audio-guide.md](reference-audio-guide.md) | How to prepare reference WAV files for voice cloning |

## Diagrams

All diagrams are written in [PlantUML](https://plantuml.com) and live in `docs/diagrams/`.
They are plain-text files and fully versionable in git.

| Diagram | File | Description |
|---------|------|-------------|
| C4 Context | [diagrams/c4-context.puml](diagrams/c4-context.puml) | Who uses the system and what it depends on |
| C4 Containers | [diagrams/c4-containers.puml](diagrams/c4-containers.puml) | Binaries, data stores, and their relationships |
| C4 Components | [diagrams/c4-components.puml](diagrams/c4-components.puml) | Rust modules inside `microverse-model` |
| Data Flow | [diagrams/data-flow.puml](diagrams/data-flow.puml) | Full pipeline from scraping to synthesized audio |
| Model Architecture | [diagrams/model-architecture.puml](diagrams/model-architecture.puml) | GPT layer stack |
| Training Sequence | [diagrams/training-sequence.puml](diagrams/training-sequence.puml) | Training loop step by step |
| Speak Sequence | [diagrams/speak-sequence.puml](diagrams/speak-sequence.puml) | End-to-end `speak` pipeline |

### Rendering diagrams locally

```bash
# Install PlantUML (Arch Linux)
sudo pacman -S plantuml

# Render a single diagram to SVG
plantuml -tsvg docs/diagrams/c4-context.puml

# Render all diagrams
plantuml -tsvg docs/diagrams/*.puml
```

SVG output is recommended — it scales cleanly and stays versionable alongside the sources.

## Milestones

| # | Title | Status |
|---|-------|--------|
| 1 | Data Ready | complete |
| 2 | Model Trains Successfully | complete |
| 3 | Quality Text Generation | complete |
| 4 | Voice Synthesis Works | in progress |
| 5 | End-to-End Demo | pending |
