use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

use clanker_config::generate_default_config;
use clanker_gateway::GatewayServer;

#[derive(Parser, Debug)]
#[command(
    name = "open-clanker",
    about = "Open Clanker AI Assistant Gateway",
    long_about = "A lightweight, Linux-optimized AI assistant gateway built in Rust. \
                  Supports multiple AI providers (Anthropic, OpenAI, Groq) and messaging channels (Telegram, Discord).",
    version = "1.0.0",
    author = "Open Clanker Contributors"
)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ConfigGenerate {
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        #[arg(short, long)]
        force: bool,
    },
    ConfigValidate {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    Gateway {
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
        #[arg(short, long, value_name = "HOST")]
        host: Option<String>,
        #[arg(short, long, value_name = "PORT")]
        port: Option<u16>,
    },
    Send {
        message: String,
        #[arg(short, long, value_name = "CHANNEL")]
        channel: Option<String>,
        #[arg(short, long, value_name = "ID")]
        chat_id: Option<String>,
    },
    Status {
        #[arg(short, long)]
        detailed: bool,
    },
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.verbose || cli.debug);

    match cli.command {
        Some(Commands::ConfigGenerate { output, force }) => cmd_config_generate(output, force).await,
        Some(Commands::ConfigValidate { config: config_path }) => cmd_config_validate(config_path.or(cli.config)).await,
        Some(Commands::Gateway { config, host, port }) => cmd_gateway(config.or(cli.config), host, port).await,
        Some(Commands::Send { message, channel, chat_id }) => cmd_send(message, channel, chat_id).await,
        Some(Commands::Status { detailed }) => cmd_status(detailed).await,
        Some(Commands::Version) => cmd_version().await,
        None => { print_welcome(); Ok(()) }
    }
}

fn setup_logging(debug: bool) {
    let level = if debug { "debug" } else { "info" };
    let log_level = tracing::Level::from_str(level).unwrap_or(tracing::Level::INFO);
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(format!("open_clanker={},clanker_core={}", level, level))
        .with_max_level(log_level)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

async fn cmd_config_generate(output: Option<PathBuf>, force: bool) -> anyhow::Result<()> {
    let output_path = output.unwrap_or_else(|| PathBuf::from("config.toml"));
    if output_path.exists() && !force {
        eprintln!("Configuration file already exists. Use --force to overwrite.");
        return Err(anyhow::anyhow!("File already exists"));
    }
    let config_content = generate_default_config();
    std::fs::write(&output_path, config_content)?;
    println!("Configuration file generated successfully!");
    println!("File: {}", output_path.display());
    Ok(())
}

async fn cmd_config_validate(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("config.toml"));
    if !config_path.exists() {
        eprintln!("Configuration file not found.");
        return Err(anyhow::anyhow!("Configuration file not found"));
    }
    println!("Validating configuration...");
    let mut config = clanker_config::Config::load_from_path(&config_path)?;
    println!("Configuration loaded successfully");

    // Load environment variables
    config.load_env()?;
    config.validate()?;
    println!("Configuration is valid!");
    Ok(())
}

async fn cmd_gateway(config_path: Option<PathBuf>, host: Option<String>, port: Option<u16>) -> anyhow::Result<()> {
    Ok(())
}

async fn cmd_send(message: String, channel: Option<String>, chat_id: Option<String>) -> anyhow::Result<()> {
    println!("Send command not yet implemented!");
    Ok(())
}

async fn cmd_status(detailed: bool) -> anyhow::Result<()> {
    println!("Open Clanker Status");
    println!("Version: open-clanker {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

async fn cmd_version() -> anyhow::Result<()> {
    println!("open-clanker {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn print_welcome() {
    println!("Open Clanker AI Assistant Gateway");
    println!("Commands: config-generate, config-validate, gateway, send, status, version");
}
