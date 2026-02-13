mod banner;
mod onboard;
mod tui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::str::FromStr;

use clanker_config::generate_default_config;
use clanker_gateway::GatewayServer;
use tokio_util::sync::CancellationToken;

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
        #[arg(long, value_name = "HOST")]
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
    Tui {
        #[arg(long, value_name = "HOST", default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, value_name = "PORT", default_value = "18789")]
        port: u16,
    },
    Onboard {
        #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
        config: PathBuf,
        #[arg(long, value_name = "FILE", default_value = ".env")]
        env_file: PathBuf,
    },
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env from current directory (for gateway, config-validate, etc.)
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Skip logging setup for TUI - it takes over the terminal
    let is_tui = matches!(cli.command, Some(Commands::Tui { .. }));
    if !is_tui {
        setup_logging(cli.verbose || cli.debug);
    }

    match cli.command {
        Some(Commands::ConfigGenerate { output, force }) => cmd_config_generate(output, force).await,
        Some(Commands::ConfigValidate { config: config_path }) => cmd_config_validate(config_path.or(cli.config)).await,
        Some(Commands::Gateway { config, host, port }) => cmd_gateway(config.or(cli.config), host, port).await,
        Some(Commands::Send { message, channel, chat_id }) => cmd_send(message, channel, chat_id).await,
        Some(Commands::Status { detailed }) => cmd_status(detailed).await,
        Some(Commands::Tui { host, port }) => cmd_tui(host, port).await,
        Some(Commands::Onboard { config, env_file }) => cmd_onboard(config, env_file).await,
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
    let config_path = config_path.unwrap_or_else(|| PathBuf::from("config.toml"));
    if !config_path.exists() {
        eprintln!("Configuration file not found: {}", config_path.display());
        eprintln!("Generate one with: open-clanker config-generate");
        return Err(anyhow::anyhow!("Configuration file not found"));
    }

    let mut config = clanker_config::Config::load_from_path(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    config.load_env().map_err(|e| anyhow::anyhow!("Failed to load env: {}", e))?;

    // Apply CLI overrides for host/port
    if let Some(h) = host {
        config.server.host = h;
    }
    if let Some(p) = port {
        config.server.port = p;
    }

    config.validate().map_err(|e| anyhow::anyhow!("Config validation failed: {}", e))?;

    let shutdown_token = CancellationToken::new();
    let server = GatewayServer::new(config, shutdown_token.clone());

    let addr = server.address();
    println!("{}", banner::gateway_banner());
    println!();
    println!("{}", "Gateway ready:".bold());
    println!("  HTTP:       http://{}", addr);
    println!("  WebSocket:  ws://{}/ws", addr);
    println!("  Health:    http://{}/health", addr);
    println!();
    println!("Press Ctrl+C to stop.");

    server.start().await.map_err(|e| anyhow::anyhow!("Gateway error: {}", e))?;

    Ok(())
}

async fn cmd_send(_message: String, _channel: Option<String>, _chat_id: Option<String>) -> anyhow::Result<()> {
    println!("Send command not yet implemented!");
    Ok(())
}

async fn cmd_status(_detailed: bool) -> anyhow::Result<()> {
    println!("Open Clanker Status");
    println!("Version: open-clanker {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

async fn cmd_onboard(config: PathBuf, env_file: PathBuf) -> anyhow::Result<()> {
    if config.exists() {
        let overwrite = dialoguer::Confirm::new()
            .with_prompt("config.toml already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("Aborted.");
            return Ok(());
        }
    }
    if env_file.exists() {
        let overwrite = dialoguer::Confirm::new()
            .with_prompt(".env already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("Aborted.");
            return Ok(());
        }
    }
    crate::onboard::run_onboard(&config, &env_file)?;
    Ok(())
}

async fn cmd_tui(host: String, port: u16) -> anyhow::Result<()> {
    println!("Connecting to gateway at {}:{}...", host, port);
    println!("Press 'q' or Esc to quit.");

    crate::tui::run_tui(&host, port).await?;
    Ok(())
}

async fn cmd_version() -> anyhow::Result<()> {
    println!("open-clanker {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn print_welcome() {
    println!("{}", banner::welcome_banner());
    println!();
    println!("{}", "Commands:".bold());
    println!("  onboard          - Interactive setup wizard (API keys, Telegram, Discord)");
    println!("  config-generate  - Generate config.toml");
    println!("  config-validate  - Validate configuration");
    println!("  gateway          - Start gateway server");
    println!("  send             - Send message (when implemented)");
    println!("  status           - Show status");
    println!("  tui              - Launch TUI client (requires gateway running)");
    println!("  version          - Show version");
    println!();
    println!("{}", "Quick start:".bold());
    println!("  ./open-clanker onboard   # or: cargo run -p clanker-cli -- onboard");
    println!("  source .env && ./open-clanker gateway");
}
