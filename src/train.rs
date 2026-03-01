use crate::dataset::Dataset;
use crate::model::{Gpt, ModelConfig};
use anyhow::Context;
use candle_core::{DType, Device, Result as CandleResult, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs;

pub struct TrainConfig {
    pub train_data: String,
    pub val_data: String,
    pub checkpoint_dir: String,
    pub max_steps: usize,
    pub lr: f64,
    pub batch_size: usize,
    pub eval_interval: usize,
    pub checkpoint_interval: usize,
    pub resume_from: Option<String>,
    pub context_len: usize,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            train_data: "datasets/train_corpus.txt".to_string(),
            val_data: "datasets/val_corpus.txt".to_string(),
            checkpoint_dir: "checkpoints".to_string(),
            max_steps: 5000,
            lr: 3e-4,
            batch_size: 32,
            eval_interval: 100,
            checkpoint_interval: 100,
            resume_from: None,
            context_len: 256,
        }
    }
}

fn cross_entropy_loss(logits: &Tensor, targets: &Tensor) -> CandleResult<Tensor> {
    let (b, t, v) = logits.dims3()?;
    candle_nn::loss::cross_entropy(&logits.reshape((b * t, v))?, &targets.reshape((b * t,))?)
}

pub fn train(config: TrainConfig) -> anyhow::Result<()> {
    let device = Device::Cpu;
    let model_cfg = ModelConfig {
        context_len: config.context_len,
        ..ModelConfig::default()
    };

    let train_ds = Dataset::from_file(&config.train_data, config.context_len)
        .with_context(|| format!("failed to load train data: {}", config.train_data))?;
    let val_ds = Dataset::from_file(&config.val_data, config.context_len)
        .with_context(|| format!("failed to load val data: {}", config.val_data))?;

    let mut varmap = VarMap::new();
    let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
    let model = Gpt::new(&model_cfg, vb).context("failed to build model")?;

    if let Some(ref path) = config.resume_from {
        varmap
            .load(path)
            .with_context(|| format!("failed to load checkpoint: {path}"))?;
        println!("Resumed from {path}");
    }

    let adam_params = ParamsAdamW {
        lr: config.lr,
        weight_decay: 0.1,
        ..Default::default()
    };
    let mut optimizer = AdamW::new(varmap.all_vars(), adam_params)?;

    fs::create_dir_all(&config.checkpoint_dir)
        .with_context(|| format!("failed to create checkpoint dir: {}", config.checkpoint_dir))?;

    let mut rng = StdRng::seed_from_u64(42);

    for step in 0..config.max_steps {
        let (inputs, targets) = train_ds
            .random_batch(config.batch_size, &mut rng, &device)
            .context("failed to generate training batch")?;

        let logits = model.forward(&inputs).context("forward pass failed")?;
        let loss = cross_entropy_loss(&logits, &targets).context("loss computation failed")?;
        optimizer
            .backward_step(&loss)
            .context("backward step failed")?;

        if step % config.eval_interval == 0 {
            let train_loss = loss
                .to_scalar::<f32>()
                .context("failed to read train loss")?;
            let val_loss = compute_val_loss(&val_ds, &model, config.batch_size, &mut rng, &device)?;
            println!(
                "step {:>5} | train_loss: {:.4} | val_loss: {:.4}",
                step, train_loss, val_loss
            );
        }

        if step % config.checkpoint_interval == 0 {
            let ckpt_path = format!("{}/ckpt_{:06}.safetensors", config.checkpoint_dir, step);
            varmap
                .save(&ckpt_path)
                .with_context(|| format!("failed to save checkpoint: {ckpt_path}"))?;
            println!("Checkpoint saved: {ckpt_path}");
        }
    }

    println!("Training complete.");
    Ok(())
}

fn compute_val_loss(
    val_ds: &Dataset,
    model: &Gpt,
    batch_size: usize,
    rng: &mut StdRng,
    device: &Device,
) -> anyhow::Result<f32> {
    let mut total = 0.0f32;
    let n = 20;
    for _ in 0..n {
        let (inputs, targets) = val_ds
            .random_batch(batch_size, rng, device)
            .context("failed to generate val batch")?;
        let logits = model.forward(&inputs).context("val forward pass failed")?;
        let loss = cross_entropy_loss(&logits, &targets).context("val loss failed")?;
        total += loss
            .to_scalar::<f32>()
            .context("failed to read val loss scalar")?;
    }
    Ok(total / n as f32)
}
