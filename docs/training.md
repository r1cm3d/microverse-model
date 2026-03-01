# Training

> Sequence diagram: [`diagrams/training-sequence.puml`](diagrams/training-sequence.puml)

Training is implemented in `src/train.rs`. The loop is intentionally straightforward: sample a random batch, forward pass, cross-entropy loss, AdamW step, repeat.

## Configuration

`TrainConfig` fields with their CLI flag names and defaults:

| Field | CLI flag | Default | Description |
|-------|----------|---------|-------------|
| `train_data` | `--train-data` | `datasets/train_corpus.txt` | Training corpus path |
| `val_data` | `--val-data` | `datasets/val_corpus.txt` | Validation corpus path |
| `checkpoint_dir` | `--checkpoint-dir` | `checkpoints` | Output directory for checkpoints |
| `max_steps` | `--max-steps` | `5000` | Total training steps |
| `lr` | `--lr` | `3e-4` | AdamW learning rate |
| `batch_size` | `--batch-size` | `32` | Sequences per batch |
| `eval_interval` | `--eval-interval` | `100` | Steps between validation runs |
| `checkpoint_interval` | `--checkpoint-interval` | `100` | Steps between checkpoint saves |
| `resume_from` | `--resume-from` | `None` | Path to `.safetensors` to resume |
| `context_len` | `--context-len` | `256` | Sequence length passed to ModelConfig |

## Optimizer

**AdamW** (`candle_nn::AdamW`):

| Parameter | Value |
|-----------|-------|
| Learning rate | 3e-4 |
| Weight decay | 0.1 |
| β₁ | 0.9 (default) |
| β₂ | 0.999 (default) |
| ε | 1e-8 (default) |

No learning rate schedule is applied. The learning rate is constant for the entire run.

## Training Loop

```
1. Load train and val datasets into memory (entire file as Vec<u8>)
2. Build Gpt model and VarMap
3. If resume_from is set: varmap.load(path) → restore all weights
4. Build AdamW over varmap.all_vars()

For step in 0..max_steps:
  a. Sample batch: random_batch(batch_size=32, context_len=256)
     inputs  = data[start..start+256]   shape (32, 256)
     targets = data[start+1..start+257] shape (32, 256)
  b. Forward: logits = model.forward(inputs)  shape (32, 256, 256)
  c. Loss: cross_entropy(logits.reshape(32×256, 256), targets.reshape(32×256))
  d. optimizer.backward_step(&loss)

  Every eval_interval steps:
    - Compute val_loss over 20 random validation batches
    - Print: "step N | train_loss: X.XXXX | val_loss: X.XXXX"

  Every checkpoint_interval steps:
    - varmap.save("checkpoints/ckpt_{step:06}.safetensors")
```

## Loss Function

Cross-entropy is computed by `candle_nn::loss::cross_entropy`, which expects:
- logits: `(N, C)` — where N = B×T and C = vocab_size
- targets: `(N,)` — flat integer class labels

The reshape before calling the loss function:

```rust
fn cross_entropy_loss(logits: &Tensor, targets: &Tensor) -> CandleResult<Tensor> {
    let (b, t, v) = logits.dims3()?;
    candle_nn::loss::cross_entropy(
        &logits.reshape((b * t, v))?,
        &targets.reshape((b * t,))?,
    )
}
```

## Checkpoints

| Property | Value |
|----------|-------|
| Format | safetensors |
| Naming | `ckpt_{step:06}.safetensors` (e.g. `ckpt_000100.safetensors`) |
| Content | All VarMap tensors (embeddings + linear weights + biases) |
| Size | ~3.5 MB per checkpoint |
| Save operation | `varmap.save(path)` |
| Load operation | `varmap.load(path)` |

Checkpoints are saved at step 0 (before any training) through to `max_steps`. To resume a run:

```bash
cargo run --release -- train --resume-from checkpoints/ckpt_000100.safetensors
```

## Expected Loss Behaviour

| Point | train_loss | Notes |
|-------|------------|-------|
| Step 0 | ~6.39 | Random init. ln(256) ≈ 5.55; slightly above that due to random weights |
| Step 100 | ~4.30 | Model learning byte-level patterns |
| Step 500+ | ~3.x | Character and word boundaries emerging |

Validation loss tracks training loss closely on a dataset this small. Significant divergence indicates overfitting.

## Resource Usage

| Resource | Value |
|----------|-------|
| Device | CPU (GPU not required) |
| Batch memory | ~512 KB per batch (32 × 256 × 4 bytes × 2 tensors) |
| Model in memory | ~10 MB (float32 weights) |
| Full run (5000 steps) | ~10–30 min on a modern CPU |
