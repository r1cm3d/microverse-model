# CLI Reference

## Binary: microverse-model

```
microverse-model [SUBCOMMAND]

Subcommands:
  train       Run the GPT training loop
  generate    Generate character dialogue from a checkpoint
  speak       Generate dialogue and synthesize audio
  help        Print help information
```

---

### train

Run the training loop. Checkpoints are saved to `checkpoints/` every `--checkpoint-interval` steps.

```
cargo run --release -- train [OPTIONS]
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--train-data` | String | `datasets/train_corpus.txt` | Training corpus |
| `--val-data` | String | `datasets/val_corpus.txt` | Validation corpus |
| `--checkpoint-dir` | String | `checkpoints` | Checkpoint output directory |
| `--max-steps` | usize | `5000` | Total training steps |
| `--lr` | f64 | `0.0003` | AdamW learning rate |
| `--batch-size` | usize | `32` | Sequences per batch |
| `--eval-interval` | usize | `100` | Steps between validation evaluations |
| `--checkpoint-interval` | usize | `100` | Steps between checkpoint saves |
| `--resume-from` | String | — | Path to `.safetensors` to resume from |
| `--context-len` | usize | `256` | Sequence context length |

**Examples:**

```bash
# Default run (5000 steps)
cargo run --release -- train

# Resume a run
cargo run --release -- train --resume-from checkpoints/ckpt_000100.safetensors

# Custom hyperparameters
cargo run --release -- train --max-steps 10000 --lr 0.001 --batch-size 16
```

---

### generate

Load a checkpoint and generate one line of dialogue. Output is written to stdout.

```
cargo run --release -- generate --checkpoint <PATH> [OPTIONS]
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--checkpoint` | String | **(required)** | Path to `.safetensors` checkpoint |
| `--character` | String | `RICK` | Character name prefix |
| `--input` | String | — | Text to continue (optional) |
| `--max-tokens` | usize | `200` | Maximum new tokens to generate |
| `--temperature` | f64 | `0.8` | Sampling temperature |
| `--top-k` | usize | `40` | Top-k cutoff (0 = disabled) |
| `--top-p` | f64 | `0.9` | Nucleus sampling threshold |
| `--seed` | u64 | `42` | Random seed |

**Examples:**

```bash
# Rick monologue
cargo run --release -- generate --checkpoint checkpoints/ckpt_005000.safetensors

# Morty responding to a prompt
cargo run --release -- generate \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --character MORTY \
  --input "Rick, I don't think this is safe"

# Deterministic greedy-like output
cargo run --release -- generate \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --temperature 0.1 --top-k 1
```

---

### speak

Generate dialogue and synthesize it to a WAV file using XTTS-v2 voice cloning.
Requires a Python virtual environment and reference audio clips.

```
cargo run --release -- speak --checkpoint <PATH> [OPTIONS]
```

All flags from `generate` apply, plus:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--output` | String | `output.wav` | Output WAV file path |
| `--samples-dir` | String | `audio_samples` | Directory with `{speaker}_reference.wav` files |
| `--script` | String | `python/tts.py` | Path to TTS Python script |
| `--python` | String | `python3` | Python interpreter path |

**Setup (required before first use):**

```bash
sudo pacman -S uv
make venv
```

**Examples:**

```bash
# Rick audio
cargo run --release -- speak \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --python .venv/bin/python3 \
  --output rick.wav

# Morty responding to a prompt
cargo run --release -- speak \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --character MORTY \
  --input "Are we gonna be okay" \
  --output morty.wav \
  --python .venv/bin/python3
```

---

## Binary: scraper

Re-scrapes all Rick and Morty episode transcripts from rickandmorty.fandom.com.

```bash
cargo run --bin scraper
# or
make scraper
```

Outputs:
- `datasets/rick_morty_all_transcripts.json`
- `datasets/rick_morty_all_transcripts.csv`

---

## Binary: preprocessor

Cleans the raw CSV and produces the training corpus.

```bash
cargo run --bin preprocessor
# or
make preprocess
```

Outputs:
- `datasets/rick_morty_transcripts_clean.csv`
- `datasets/train_corpus.txt`
- `datasets/val_corpus.txt`

---

## Makefile Targets

Run `make help` to see all targets.

| Target | Description |
|--------|-------------|
| `make build` | `cargo build` (dev profile) |
| `make release` | `cargo build --release` |
| `make test` | `cargo test` |
| `make clippy` | `cargo clippy -- -D warnings` |
| `make fmt` | `cargo fmt` |
| `make doc` | `cargo doc --no-deps --open` |
| `make clean` | `cargo clean` |
| `make scraper` | Run transcript scraper |
| `make preprocess` | Run data preprocessor |
| `make train` | `cargo run --release -- train` |
| `make generate CHECKPOINT=<path>` | Run text generation |
| `make speak CHECKPOINT=<path>` | Run voice synthesis |
| `make venv` | Create `.venv` and install `python/requirements.txt` |

### Makefile variables

Override on the command line: `make speak CHECKPOINT=... CHARACTER=MORTY OUTPUT=out.wav`

| Variable | Default | Used by |
|----------|---------|---------|
| `CHECKPOINT` | *(empty)* | `generate`, `speak` |
| `CHARACTER` | `RICK` | `generate`, `speak` |
| `INPUT` | *(empty)* | `generate`, `speak` |
| `OUTPUT` | `output.wav` | `speak` |
| `PYTHON` | `.venv/bin/python3` | `speak` |
