use clap::{Parser, Subcommand};
use microverse_model::train::{TrainConfig, train};

#[derive(Parser)]
#[command(name = "microverse-model", about = "Microverse SLM training CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Train(TrainArgs),
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
    }
}
