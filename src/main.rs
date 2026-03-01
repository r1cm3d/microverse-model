use clap::{Parser, Subcommand};
use microverse_model::generate::{generate, generate_text, GenerateConfig};
use microverse_model::train::{train, TrainConfig};
use microverse_model::tts_bridge::{speak, TtsBridgeConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "microverse-model", about = "Microverse SLM training CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Train(TrainArgs),
    Generate(GenerateArgs),
    Speak(SpeakArgs),
}

#[derive(Parser)]
struct GenerateArgs {
    #[arg(long)]
    checkpoint: String,

    #[arg(long, default_value = "RICK")]
    character: String,

    #[arg(long)]
    input: Option<String>,

    #[arg(long = "max-tokens", default_value_t = 200)]
    max_new_tokens: usize,

    #[arg(long, default_value_t = 0.8)]
    temperature: f64,

    #[arg(long = "top-k", default_value_t = 40)]
    top_k: usize,

    #[arg(long = "top-p", default_value_t = 0.9)]
    top_p: f64,

    #[arg(long, default_value_t = 42)]
    seed: u64,
}

#[derive(Parser)]
struct SpeakArgs {
    #[arg(long)]
    checkpoint: String,

    #[arg(long, default_value = "RICK")]
    character: String,

    #[arg(long)]
    input: Option<String>,

    #[arg(long = "max-tokens", default_value_t = 200)]
    max_new_tokens: usize,

    #[arg(long, default_value_t = 0.8)]
    temperature: f64,

    #[arg(long = "top-k", default_value_t = 40)]
    top_k: usize,

    #[arg(long = "top-p", default_value_t = 0.9)]
    top_p: f64,

    #[arg(long, default_value_t = 42)]
    seed: u64,

    #[arg(long, default_value = "output.wav")]
    output: String,

    #[arg(long, default_value = "audio_samples")]
    samples_dir: String,

    #[arg(long, default_value = "python/tts.py")]
    script: String,

    #[arg(long, default_value = "python3")]
    python: String,
}

#[derive(Parser)]
struct TrainArgs {
    #[arg(long, default_value = "datasets/train_corpus.txt")]
    train_data: String,

    #[arg(long, default_value = "datasets/val_corpus.txt")]
    val_data: String,

    #[arg(long, default_value = "checkpoints")]
    checkpoint_dir: String,

    #[arg(long, default_value_t = 5000)]
    max_steps: usize,

    #[arg(long, default_value_t = 3e-4)]
    lr: f64,

    #[arg(long, default_value_t = 32)]
    batch_size: usize,

    #[arg(long)]
    resume_from: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Train(args) => {
            let config = TrainConfig {
                train_data: args.train_data,
                val_data: args.val_data,
                checkpoint_dir: args.checkpoint_dir,
                max_steps: args.max_steps,
                lr: args.lr,
                batch_size: args.batch_size,
                resume_from: args.resume_from,
                ..TrainConfig::default()
            };
            train(config)
        }
        Commands::Generate(args) => {
            let config = GenerateConfig {
                checkpoint: args.checkpoint,
                character: args.character,
                input: args.input,
                max_new_tokens: args.max_new_tokens,
                temperature: args.temperature,
                top_k: args.top_k,
                top_p: args.top_p,
                seed: args.seed,
            };
            generate(config)
        }
        Commands::Speak(args) => {
            let gen_config = GenerateConfig {
                checkpoint: args.checkpoint,
                character: args.character.clone(),
                input: args.input,
                max_new_tokens: args.max_new_tokens,
                temperature: args.temperature,
                top_k: args.top_k,
                top_p: args.top_p,
                seed: args.seed,
            };
            let text = generate_text(gen_config)?;
            println!("{}", text);

            let tts_config = TtsBridgeConfig {
                text,
                character: args.character,
                output: PathBuf::from(&args.output),
                script: PathBuf::from(&args.script),
                samples_dir: PathBuf::from(&args.samples_dir),
                python: args.python,
            };
            let path = speak(tts_config)?;
            println!("Audio saved to: {}", path.display());
            Ok(())
        }
    }
}
