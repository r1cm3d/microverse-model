# Text Generation

Generation is implemented in `src/generate.rs`. The model runs autoregressively: each new token is sampled from the distribution over the vocabulary and appended to the context, which is then fed back into the model for the next step.

## Configuration

`GenerateConfig` fields with their CLI flag names and defaults:

| Field | CLI flag | Default | Description |
|-------|----------|---------|-------------|
| `checkpoint` | `--checkpoint` | (required) | Path to `.safetensors` checkpoint |
| `character` | `--character` | `RICK` | Character name prefix |
| `input` | `--input` | `None` | Optional text to continue |
| `max_new_tokens` | `--max-tokens` | `200` | Maximum tokens to generate |
| `temperature` | `--temperature` | `0.8` | Sampling temperature |
| `top_k` | `--top-k` | `40` | Top-k cutoff (0 = disabled) |
| `top_p` | `--top-p` | `0.9` | Nucleus sampling threshold |
| `seed` | `--seed` | `42` | Random seed for reproducibility |

## Prompt Format

The generation prompt is constructed as:

```
"CHARACTER: input_text "       (with trailing space, when --input is provided)
"CHARACTER: "                   (when no --input is provided)
```

Example prompts:
```
"RICK: "
"MORTY: Where are we going "
```

The prompt is byte-encoded and used as the initial context. Only tokens generated after the prompt are returned in the output.

## Autoregressive Loop

```
1. encode(prompt) → Vec<u8> → Vec<u32>  (initial context)

2. for _ in 0..max_new_tokens:
     window = context[max(0, len-context_len)..]   // last 256 tokens
     input  = Tensor(window, shape=(1, T))

     logits = model.forward(input)                  // (1, T, 256)
     last   = logits[0, T-1, :]                     // (256,) last position

     last /= temperature                            // scale
     apply_top_k(&mut last, top_k)                 // mask bottom vocab
     apply_top_p(&mut last, top_p)                 // nucleus mask
     token = weighted_sample(last, &mut rng)        // draw token

     context.push(token)
     if token == b'\n' { break }

3. generated = context[prompt_len..]
4. return decode(generated)
```

The loop stops on a newline token (a complete dialogue line has been generated) or when `max_new_tokens` is reached.

## Sampling Strategies

### Temperature Scaling

Divides all logits by `temperature` before computing probabilities:

```
scaled[i] = logit[i] / temperature
```

- `temperature < 1.0` → sharper distribution → more confident/repetitive
- `temperature = 1.0` → raw model distribution
- `temperature > 1.0` → flatter distribution → more diverse/random

Default: `0.8` (slightly sharpened).

### Top-K Filtering

Keeps only the `k` highest-probability tokens; sets all others to `-∞`:

```
1. sort logits descending
2. threshold = logits[k-1]
3. for each logit: if logit < threshold → set to -∞
```

With `k = 0`, the filter is disabled. Default: `k = 40`.

### Top-P (Nucleus) Sampling

Keeps the smallest set of tokens whose cumulative probability exceeds `p`:

```
1. compute softmax probabilities (for ranking only)
2. sort tokens by probability descending
3. accumulate probabilities until cumulative sum >= p
4. set remaining logits to -∞
```

With `p = 1.0`, the filter is disabled. Default: `p = 0.9`.

### Weighted Sampling

After filtering, a token is drawn from the remaining distribution:

```
weights[i] = exp(logit[i] - max_logit)   // numerically stable
token = WeightedIndex(weights).sample(rng)
```

## API

| Function | Signature | Use case |
|----------|-----------|----------|
| `generate_text` | `(GenerateConfig) → Result<String>` | Returns the generated string. Used by the `speak` pipeline. |
| `generate` | `(GenerateConfig) → Result<()>` | Prints generated text to stdout. Used by the `generate` subcommand. |

## Examples

```bash
# Default (RICK, no input)
cargo run --release -- generate --checkpoint checkpoints/ckpt_005000.safetensors

# Morty with an input prompt
cargo run --release -- generate \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --character MORTY \
  --input "Are we gonna be okay"

# Greedy-like (low temperature, small k)
cargo run --release -- generate \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --temperature 0.3 \
  --top-k 5 \
  --top-p 1.0

# High diversity
cargo run --release -- generate \
  --checkpoint checkpoints/ckpt_005000.safetensors \
  --temperature 1.2 \
  --top-k 0 \
  --top-p 0.95
```
