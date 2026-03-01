use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{Embedding, LayerNorm, Linear, Module, VarBuilder, embedding, layer_norm, linear};

#[derive(Debug, Clone, Copy)]
pub struct ModelConfig {
    pub vocab_size: usize,
    pub d_model: usize,
    pub n_heads: usize,
    pub n_layers: usize,
    pub context_len: usize,
    pub d_ff: usize,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            vocab_size: 256,
            d_model: 128,
            n_heads: 4,
            n_layers: 4,
            context_len: 256,
            d_ff: 512,
        }
    }
}

struct CausalSelfAttention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    out_proj: Linear,
    n_heads: usize,
    head_dim: usize,
}

impl CausalSelfAttention {
    fn new(cfg: &ModelConfig, vb: VarBuilder) -> Result<Self> {
        let head_dim = cfg.d_model / cfg.n_heads;
        Ok(Self {
            q_proj: linear(cfg.d_model, cfg.d_model, vb.pp("q_proj"))?,
            k_proj: linear(cfg.d_model, cfg.d_model, vb.pp("k_proj"))?,
            v_proj: linear(cfg.d_model, cfg.d_model, vb.pp("v_proj"))?,
            out_proj: linear(cfg.d_model, cfg.d_model, vb.pp("out_proj"))?,
            n_heads: cfg.n_heads,
            head_dim,
        })
    }

    fn forward(&self, x: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let (b, t, _) = x.dims3()?;
        let q = self.q_proj.forward(x)?;
        let k = self.k_proj.forward(x)?;
        let v = self.v_proj.forward(x)?;

        let q = q
            .reshape((b, t, self.n_heads, self.head_dim))?
            .transpose(1, 2)?
            .contiguous()?;
        let k = k
            .reshape((b, t, self.n_heads, self.head_dim))?
            .transpose(1, 2)?
            .contiguous()?;
        let v = v
            .reshape((b, t, self.n_heads, self.head_dim))?
            .transpose(1, 2)?
            .contiguous()?;

        let scale = (self.head_dim as f64).sqrt();
        let scores = (q.matmul(&k.transpose(2, 3)?)? / scale)?;

        let mask_slice = mask.narrow(0, 0, t)?.narrow(1, 0, t)?;
        let scores = scores.broadcast_add(&mask_slice.unsqueeze(0)?.unsqueeze(0)?)?;

        let attn = candle_nn::ops::softmax(&scores, candle_core::D::Minus1)?;
        let out = attn
            .matmul(&v)?
            .transpose(1, 2)?
            .contiguous()?
            .reshape((b, t, self.n_heads * self.head_dim))?;

        self.out_proj.forward(&out)
    }
}

struct FeedForward {
    fc1: Linear,
    fc2: Linear,
}

impl FeedForward {
    fn new(cfg: &ModelConfig, vb: VarBuilder) -> Result<Self> {
        Ok(Self {
            fc1: linear(cfg.d_model, cfg.d_ff, vb.pp("fc1"))?,
            fc2: linear(cfg.d_ff, cfg.d_model, vb.pp("fc2"))?,
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        self.fc2.forward(&self.fc1.forward(x)?.gelu()?)
    }
}

struct TransformerBlock {
    ln1: LayerNorm,
    attn: CausalSelfAttention,
    ln2: LayerNorm,
    ff: FeedForward,
}

impl TransformerBlock {
    fn new(cfg: &ModelConfig, vb: VarBuilder) -> Result<Self> {
        Ok(Self {
            ln1: layer_norm(cfg.d_model, 1e-5, vb.pp("ln1"))?,
            attn: CausalSelfAttention::new(cfg, vb.pp("attn"))?,
            ln2: layer_norm(cfg.d_model, 1e-5, vb.pp("ln2"))?,
            ff: FeedForward::new(cfg, vb.pp("ff"))?,
        })
    }

    fn forward(&self, x: &Tensor, mask: &Tensor) -> Result<Tensor> {
        let x = (x + self.attn.forward(&self.ln1.forward(x)?, mask)?)?;
        let x = (&x + self.ff.forward(&self.ln2.forward(&x)?)?)?;
        Ok(x)
    }
}

pub struct Gpt {
    token_emb: Embedding,
    pos_emb: Embedding,
    blocks: Vec<TransformerBlock>,
    ln_f: LayerNorm,
    lm_head: Linear,
    causal_mask: Tensor,
}

impl Gpt {
    pub fn new(cfg: &ModelConfig, vb: VarBuilder) -> Result<Self> {
        let token_emb = embedding(cfg.vocab_size, cfg.d_model, vb.pp("token_emb"))?;
        let pos_emb = embedding(cfg.context_len, cfg.d_model, vb.pp("pos_emb"))?;

        let blocks = (0..cfg.n_layers)
            .map(|i| TransformerBlock::new(cfg, vb.pp(format!("block_{i}"))))
            .collect::<Result<Vec<_>>>()?;

        let ln_f = layer_norm(cfg.d_model, 1e-5, vb.pp("ln_f"))?;
        let lm_head = linear(cfg.d_model, cfg.vocab_size, vb.pp("lm_head"))?;

        let mask = build_causal_mask(cfg.context_len, vb.device())?;

        Ok(Self {
            token_emb,
            pos_emb,
            blocks,
            ln_f,
            lm_head,
            causal_mask: mask,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let (_, t) = input.dims2()?;
        let device = input.device();

        let tok = self.token_emb.forward(input)?;
        let pos_ids = Tensor::arange(0u32, t as u32, device)?;
        let pos = self.pos_emb.forward(&pos_ids)?;
        let mut x = tok.broadcast_add(&pos)?;

        for block in &self.blocks {
            x = block.forward(&x, &self.causal_mask)?;
        }

        self.lm_head.forward(&self.ln_f.forward(&x)?)
    }
}

fn build_causal_mask(context_len: usize, device: &Device) -> Result<Tensor> {
    let mask: Vec<f32> = (0..context_len)
        .flat_map(|row| {
            (0..context_len).map(move |col| if col > row { -1e9f32 } else { 0.0f32 })
        })
        .collect();
    Tensor::from_vec(mask, (context_len, context_len), device)?.to_dtype(DType::F32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    use candle_nn::VarMap;

    #[test]
    fn test_gpt_forward_shape() {
        let cfg = ModelConfig::default();
        let device = Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let model = Gpt::new(&cfg, vb).unwrap();

        let batch_size = 2;
        let seq_len = 16usize;
        let input = Tensor::zeros((batch_size, seq_len), DType::U32, &device).unwrap();
        let logits = model.forward(&input).unwrap();

        assert_eq!(logits.dims(), &[batch_size, seq_len, cfg.vocab_size]);
    }
}
