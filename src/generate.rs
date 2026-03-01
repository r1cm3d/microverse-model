use crate::model::{Gpt, ModelConfig};
use crate::tokenizer;
use anyhow::Context;
use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{VarBuilder, VarMap};
use rand::distributions::{Distribution, WeightedIndex};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::cmp::Ordering;

pub struct GenerateConfig {
    pub checkpoint: String,
    pub character: String,
    pub input: Option<String>,
    pub max_new_tokens: usize,
    pub temperature: f64,
    pub top_k: usize,
    pub top_p: f64,
    pub seed: u64,
}

pub fn generate_text(config: GenerateConfig) -> anyhow::Result<String> {
    let device = Device::Cpu;
    let model_cfg = ModelConfig::default();

    let mut varmap = VarMap::new();
    let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
    let model = Gpt::new(&model_cfg, vb).context("failed to build model")?;
    varmap
        .load(&config.checkpoint)
        .with_context(|| format!("failed to load checkpoint: {}", config.checkpoint))?;

    let prompt = match &config.input {
        Some(input) => format!("{}: {} ", config.character, input),
        None => format!("{}: ", config.character),
    };

    let prompt_bytes = tokenizer::encode(&prompt);
    let prompt_len = prompt_bytes.len();
    let mut context: Vec<u32> = prompt_bytes.iter().map(|&b| b as u32).collect();

    let mut rng = StdRng::seed_from_u64(config.seed);
    let context_len = model_cfg.context_len;

    for _ in 0..config.max_new_tokens {
        let t = context.len().min(context_len);
        let window: Vec<u32> = context[context.len() - t..].to_vec();

        let input =
            Tensor::from_vec(window, (1, t), &device).context("failed to create input tensor")?;
        let logits = model.forward(&input).context("forward pass failed")?;

        let last_logits = logits.i((0, t - 1, ..))?;
        let last_logits = last_logits.to_dtype(DType::F32)?;
        let mut logit_vec = last_logits
            .to_vec1::<f32>()
            .context("failed to extract logits")?;

        for l in &mut logit_vec {
            *l /= config.temperature as f32;
        }

        if config.top_k > 0 {
            apply_top_k(&mut logit_vec, config.top_k);
        }

        if config.top_p < 1.0 {
            apply_top_p(&mut logit_vec, config.top_p);
        }

        let token = sample_from_logits(logit_vec, &mut rng)?;
        context.push(token);

        if token == b'\n' as u32 {
            break;
        }
    }

    let generated_bytes: Vec<u8> = context[prompt_len..].iter().map(|&t| t as u8).collect();
    Ok(tokenizer::decode(&generated_bytes))
}

pub fn generate(config: GenerateConfig) -> anyhow::Result<()> {
    let text = generate_text(config)?;
    print!("{}", text);
    Ok(())
}

fn apply_top_k(logits: &mut [f32], k: usize) {
    if k == 0 || k >= logits.len() {
        return;
    }
    let mut sorted = logits.to_owned();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
    let threshold = sorted[k - 1];
    for l in logits.iter_mut() {
        if *l < threshold {
            *l = f32::NEG_INFINITY;
        }
    }
}

fn apply_top_p(logits: &mut [f32], p: f64) {
    let n = logits.len();
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| logits[b].partial_cmp(&logits[a]).unwrap_or(Ordering::Equal));

    let max_val = indices
        .iter()
        .map(|&i| logits[i])
        .fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = (0..n).map(|i| (logits[i] - max_val).exp()).collect();
    let sum: f32 = exps.iter().sum();

    let mut cumulative = 0.0f64;
    let mut cutoff_found = false;
    for &idx in &indices {
        if cutoff_found {
            logits[idx] = f32::NEG_INFINITY;
        } else {
            cumulative += (exps[idx] / sum) as f64;
            if cumulative >= p {
                cutoff_found = true;
            }
        }
    }
}

fn sample_from_logits(logits: Vec<f32>, rng: &mut StdRng) -> anyhow::Result<u32> {
    let max_val = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let weights: Vec<f32> = logits.iter().map(|&l| (l - max_val).exp()).collect();
    let dist = WeightedIndex::new(&weights).context("failed to create weighted distribution")?;
    Ok(dist.sample(rng) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_top_k_zeroes_below_threshold() {
        let mut logits = vec![1.0f32, 5.0, 3.0, 2.0, 4.0];
        apply_top_k(&mut logits, 2);
        assert!(logits[1] > f32::NEG_INFINITY);
        assert!(logits[4] > f32::NEG_INFINITY);
        assert_eq!(logits[0], f32::NEG_INFINITY);
        assert_eq!(logits[2], f32::NEG_INFINITY);
        assert_eq!(logits[3], f32::NEG_INFINITY);
    }

    #[test]
    fn test_apply_top_k_no_op_when_k_zero() {
        let original = vec![1.0f32, 5.0, 3.0, 2.0, 4.0];
        let mut logits = original.clone();
        apply_top_k(&mut logits, 0);
        assert_eq!(logits, original);
    }

    #[test]
    fn test_apply_top_p_zeroes_tail() {
        let mut logits = vec![1.0f32; 10];
        apply_top_p(&mut logits, 0.5);
        let non_neg_inf = logits.iter().filter(|&&l| l > f32::NEG_INFINITY).count();
        assert_eq!(non_neg_inf, 5);
    }

    #[test]
    fn test_sample_from_logits_valid_index() {
        let logits = vec![0.0f32, 10.0, 0.0];
        let mut rng = StdRng::seed_from_u64(0);
        let token = sample_from_logits(logits, &mut rng).unwrap();
        assert_eq!(token, 1);
    }
}
