# Model Architecture

> Source: [`diagrams/model-architecture.puml`](diagrams/model-architecture.puml)

The model is a standard GPT-style decoder-only transformer implemented in `src/model.rs` using the `candle` framework.

## Configuration

`ModelConfig` holds all hyperparameters. The `Default` impl provides these values:

| Field | Default | Description |
|-------|---------|-------------|
| `vocab_size` | 256 | One token per byte value (byte-level tokenizer) |
| `d_model` | 128 | Embedding dimension |
| `n_heads` | 4 | Number of attention heads |
| `n_layers` | 4 | Number of stacked TransformerBlocks |
| `context_len` | 256 | Maximum sequence length (tokens) |
| `d_ff` | 512 | Feed-forward hidden dimension (4× d_model) |

Derived: `head_dim = d_model / n_heads = 32`.

Total parameters: approximately 2.6M.

## Layer Stack

```
Input  (B × T)  token IDs
  │
  ├─ Token Embedding      [256 × 128]
  ├─ Positional Embedding [256 × 128]
  └─ ⊕ broadcast_add  →  (B × T × 128)
         │
         ▼
  ┌─────────────────────────────────────────┐
  │  TransformerBlock  × 4                  │
  │                                         │
  │  LayerNorm (pre-norm)                   │
  │  CausalSelfAttention                    │
  │  ⊕ residual                             │
  │                                         │
  │  LayerNorm (pre-norm)                   │
  │  FeedForward  (GELU)                    │
  │  ⊕ residual                             │
  └─────────────────────────────────────────┘
         │
  Final LayerNorm
         │
  LM Head  [128 × 256]  (no bias)
         │
Output  (B × T × 256)  logits
```

## CausalSelfAttention

Each attention layer projects inputs to Q, K, V spaces and computes multi-head scaled dot-product attention.

**Projections** (all Linear with bias):
- `q_proj`: 128 → 128
- `k_proj`: 128 → 128
- `v_proj`: 128 → 128
- `out_proj`: 128 → 128

**Forward pass:**

```
1.  Q = q_proj(x)   →  reshape (B, n_heads, T, head_dim)
    K = k_proj(x)   →  reshape (B, n_heads, T, head_dim)
    V = v_proj(x)   →  reshape (B, n_heads, T, head_dim)

2.  scores = Q · Kᵀ / √head_dim           shape: (B, n_heads, T, T)

3.  scores += causal_mask[0..T, 0..T]     mask[row,col] = -1e9 if col > row
                                                                  0    otherwise

4.  attn = softmax(scores, dim=-1)

5.  out = attn · V                         shape: (B, n_heads, T, head_dim)
        → transpose (1,2) → contiguous → reshape (B, T, d_model)

6.  return out_proj(out)
```

The causal mask is pre-built once at model construction time as a `(context_len × context_len)` tensor, then sliced to `(T × T)` each forward call to support variable-length inputs during generation.

**Key candle pattern:** after `transpose`, always call `.contiguous()` before `reshape` to avoid non-contiguous memory layout errors.

## FeedForward

A two-layer MLP with GELU activation and 4× expansion ratio:

```
x → fc1 [128 → 512] → GELU → fc2 [512 → 128] → output
```

Both `fc1` and `fc2` are Linear layers with bias.

## TransformerBlock

Uses **pre-LayerNorm** (norm before sublayer, not after). This improves gradient flow during training compared to the original post-norm formulation:

```rust
// Attention sublayer
let residual = x.clone();
let x = ln1.forward(x)?;         // normalize first
let x = attn.forward(x, mask)?;
let x = (x + residual)?;         // then add residual

// Feed-forward sublayer
let residual = x.clone();
let x = ln2.forward(x)?;
let x = ff.forward(x)?;
let x = (x + residual)?;
```

## Positional Embedding

Learned absolute positional embeddings. At each forward pass, position IDs are generated with:

```rust
Tensor::arange(0u32, t as u32, device)?
```

These are looked up in `pos_emb` and broadcast-added to the token embeddings. No sinusoidal encoding is used.

## Parameter Breakdown

| Component | Shape | Parameters |
|-----------|-------|------------|
| Token embedding | 256 × 128 | 32,768 |
| Positional embedding | 256 × 128 | 32,768 |
| Per TransformerBlock | — | ~263,168 |
| — q/k/v/out projections | 4 × (128×128 + 128) | 66,048 |
| — ff fc1 + fc2 | (128×512+512) + (512×128+128) | 131,712 |
| — 2× LayerNorm | 2 × (128 + 128) | 512 |
| Final LayerNorm | 2 × 128 | 256 |
| LM Head | 128 × 256 (no bias) | 32,768 |
| **Total (4 blocks)** | | **~2.18M** |

Checkpoint size: ~3.5 MB (float32 weights + safetensors metadata).
