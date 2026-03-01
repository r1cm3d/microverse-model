# Voice Synthesis

> Sequence diagram: [`diagrams/speak-sequence.puml`](diagrams/speak-sequence.puml)

The `speak` subcommand runs the full pipeline: generate text with the trained model, then synthesize audio using XTTS-v2 voice cloning. The two phases are independently callable — text generation is in `src/generate.rs`, and TTS is in `src/tts_bridge.rs` + `python/tts.py`.

## Architecture

```
microverse-model (speak subcommand)
  │
  ├─ generate_text(GenerateConfig)     → generated dialogue string
  │       (src/generate.rs)
  │
  └─ speak(TtsBridgeConfig)            → PathBuf to output WAV
          (src/tts_bridge.rs)
                │
                └─ spawn subprocess: python tts.py
                        (python/tts.py)
                               │
                               ├─ reads: audio_samples/{speaker}_reference.wav
                               └─ runs: XTTS-v2 inference → output.wav
```

## TTS Bridge Configuration

`TtsBridgeConfig` fields:

| Field | CLI flag | Default | Description |
|-------|----------|---------|-------------|
| `text` | — | (from generate_text) | Text to synthesize |
| `character` | `--character` | `RICK` | Speaker name (converted to lowercase for the script) |
| `output` | `--output` | `output.wav` | Output WAV file path |
| `script` | `--script` | `python/tts.py` | Path to the TTS Python script |
| `samples_dir` | `--samples-dir` | `audio_samples` | Directory containing reference clips |
| `python` | `--python` | `python3` | Python interpreter (e.g. `.venv/bin/python3`) |

## Python TTS Script

`python/tts.py` is a minimal wrapper around `TTS.api.TTS`:

```python
tts = TTS(model_name="tts_models/multilingual/multi-dataset/xtts_v2")
tts.tts_to_file(
    text=args.text,
    speaker_wav=ref_path,      # audio_samples/{speaker}_reference.wav
    language="en",
    file_path=args.output,
)
```

XTTS-v2 downloads its weights automatically on first use (~1.8 GB). On subsequent runs the model is cached locally by the TTS library.

## Setting Up the Python Environment

```bash
# Install uv (Arch Linux)
sudo pacman -S uv

# Create .venv with Python 3.11 and install TTS
make venv

# Verify
.venv/bin/python3 -c "from TTS.api import TTS; print('ok')"
```

The Makefile `speak` target uses `.venv/bin/python3` by default. Override with:

```bash
make speak CHECKPOINT=... PYTHON=/usr/bin/python3
```

## Reference Audio Requirements

XTTS-v2 voice cloning quality depends heavily on the reference clip. Required:

| Property | Requirement |
|----------|-------------|
| Duration | 3–15 seconds |
| Sample rate | 22050 Hz |
| Channels | Mono |
| Format | WAV (PCM) |
| Content | Single speaker, no music, no sound effects |

Expected file paths:
```
audio_samples/rick_reference.wav
audio_samples/morty_reference.wav
```

For instructions on downloading and preparing these clips, see [reference-audio-guide.md](reference-audio-guide.md).

## Error Handling

`speak()` in `tts_bridge.rs` validates the subprocess result at two points:

1. **Non-zero exit code** — collects stderr and returns an error with the full Python traceback.
2. **Output file missing** — returns an error even if the script exits 0 (guards against silent failures in TTS library internals).

```rust
if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("tts.py exited with error:\n{}", stderr);
}

if !config.output.exists() {
    anyhow::bail!("tts.py exited 0 but output file not found: {}", config.output.display());
}
```

## Performance

| Metric | Value |
|--------|-------|
| Model download (first run) | ~1.8 GB |
| Synthesis time (CPU) | 30–60 s per utterance |
| Output sample rate | 24000 Hz (XTTS-v2 output) |
| Output format | WAV (PCM, 16-bit) |

GPU acceleration is supported by XTTS-v2 but not required. To use a GPU, install the appropriate PyTorch CUDA build before running `make venv`.

## End-to-End Example

```bash
# Generate and synthesize a Rick line
make speak \
  CHECKPOINT=checkpoints/ckpt_005000.safetensors \
  CHARACTER=RICK \
  OUTPUT=rick_output.wav

# With input text
make speak \
  CHECKPOINT=checkpoints/ckpt_005000.safetensors \
  CHARACTER=MORTY \
  INPUT="Are we gonna be okay, Rick" \
  OUTPUT=morty_response.wav
```

Expected terminal output:

```
It's science, Morty! You can't just ignore science!
Synthesizing audio (this may take ~30-60s on CPU)...
Audio saved to: rick_output.wav
```
