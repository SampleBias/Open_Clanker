# Quick Start Guide: From OpenClaw to Open Clanker

## Executive Summary

This guide provides the **concrete first steps** to begin migrating OpenClaw from TypeScript to Rust, creating a lightweight, Linux-optimized, Docker-ready AI assistant gateway.

---

## Phase 1: Initial Setup (Day 1)

### Step 1: Create Rust Workspace Structure

```bash
# Navigate to project root
cd /home/s4mpl3bi4s/rusty_clanker/openclaw-main

# Create Rust workspace structure
mkdir -p crates/{core,gateway,agent,channels,cli,config,storage}
mkdir -p docker
mkdir -p docs
mkdir -p config-examples
```

### Step 2: Create Workspace Cargo.toml

Create `Cargo.toml` at root:

```toml
[workspace]
members = [
    "crates/core",
    "crates/gateway",
    "crates/agent",
    "crates/channels",
    "crates/cli",
    "crates/config",
    "crates/storage",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["Open Clanker Contributors"]
license = "MIT"
repository = "https://github.com/openclanker/open-clanker"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-util = "0.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Web framework
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# WebSocket
tokio-tungstenite = "0.21"
futures-util = "0.3"

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# CLI
clap = { version = "4.5", features = ["derive", "env"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
once_cell = "1.19"
dashmap = "5.5"

# Telegram bot
teloxide = { version = "0.12", features = ["macros"] }

# Discord bot
serenity = { version = "0.12", default-features = false, features = ["client", "gateway", "rustls_backend"] }

# Async traits
async-trait = "0.1"
```

---

## Phase 2: Core Types (Day 1-2)

### Step 1: Create Core Crate

Create `crates/core/Cargo.toml`:

```toml
[package]
name = "clanker-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
async-trait = { workspace = true }
```

### Step 2: Implement Core Types

Create `crates/core/src/types.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for messages
pub type MessageId = String;

/// User/channel identifier
pub type UserId = String;

/// Message sent through the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub channel_type: ChannelType,
    pub channel_id: String,
    pub sender: UserId,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: MessageMetadata,
}

impl Message {
    pub fn new(
        channel_type: ChannelType,
        channel_id: String,
        sender: String,
        text: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            channel_type,
            channel_id,
            sender,
            text,
            timestamp: Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageMetadata {
    pub attachments: Vec<Attachment>,
    pub reply_to: Option<MessageId>,
    pub mentions: Vec<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub url: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

/// Supported channel types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Telegram,
    Discord,
}

/// Agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub content: String,
    pub model: String,
    pub finish_reason: String,
    pub usage: UsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

Create `crates/core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClankerError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Channel error [{channel}]: {message}")]
    Channel { channel: String, message: String },

    #[error("Agent error [{provider}]: {message}")]
    Agent { provider: String, message: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Authentication failed")]
    Authentication,

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, ClankerError>;
```

Create `crates/core/src/traits.rs`:

```rust
use crate::types::Message;

/// Trait for message channels
#[async_trait::async_trait]
pub trait Channel: Send + Sync {
    /// Channel identifier
    fn channel_type(&self) -> crate::types::ChannelType;

    /// Send a message
    async fn send_message(&self, message: &Message) -> crate::Result<()>;

    /// Start listening for messages
    async fn listen(&self) -> crate::Result<()>;

    /// Stop listening
    async fn stop(&self) -> crate::Result<()>;
}

/// Trait for AI agents
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// Generate a response
    async fn generate(&self, messages: &[Message]) -> crate::Result<crate::types::AgentResponse>;

    /// Check health
    async fn health(&self) -> crate::Result<bool>;
}
```

Create `crates/core/src/lib.rs`:

```rust
pub mod error;
pub mod traits;
pub mod types;

pub use error::{ClankerError, Result};
pub use traits::{Agent, Channel};
pub use types::*;
```

---

## Phase 3: Configuration (Day 2)

### Step 1: Create Config Crate

Create `crates/config/Cargo.toml`:

```toml
[package]
name = "clanker-config"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
toml = { workspace = true }
thiserror = { workspace = true }
clanker-core = { path = "../core" }
```

### Step 2: Implement Configuration

Create `crates/config/src/lib.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub channels: ChannelsConfig,
    pub agent: AgentConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelsConfig {
    pub telegram: Option<TelegramConfig>,
    pub discord: Option<DiscordConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chats: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub guild_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AgentConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_env(&mut self) {
        if let Some(telegram) = &mut self.channels.telegram {
            if let Ok(token) = std::env::var("OPENCLAW_TELEGRAM_BOT_TOKEN") {
                telegram.bot_token = token;
            }
        }

        if let Some(discord) = &mut self.channels.discord {
            if let Ok(token) = std::env::var("OPENCLAW_DISCORD_BOT_TOKEN") {
                discord.bot_token = token;
            }
        }
    }
}

pub fn generate_default_config() -> String {
    toml::to_string(&Config {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 18789,
            tls: None,
        },
        channels: ChannelsConfig {
            telegram: Some(TelegramConfig {
                bot_token: "your-telegram-bot-token".to_string(),
                allowed_chats: None,
            }),
            discord: Some(DiscordConfig {
                bot_token: "your-discord-bot-token".to_string(),
                guild_id: None,
            }),
        },
        agent: AgentConfig {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key_env: "OPENCLAW_ANTHROPIC_API_KEY".to_string(),
            max_tokens: 4096,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        },
    })
    .unwrap()
}
```

### Step 3: Create Example Config

Create `config-examples/config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 18789

[channels.telegram]
bot_token = "your-telegram-bot-token"
allowed_chats = []

[channels.discord]
bot_token = "your-discord-bot-token"
guild_id = null

[agent]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "OPENCLAW_ANTHROPIC_API_KEY"
max_tokens = 4096

[logging]
level = "info"
format = "json"
```

---

## Phase 4: Gateway Server (Day 3-4)

### Step 1: Create Gateway Crate

Create `crates/gateway/Cargo.toml`:

```toml
[package]
name = "clanker-gateway"
version.workspace = true
edition.workspace = true

[dependencies]
axum = { workspace = true }
tokio = { workspace = true }
tokio-tungstenite = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
clanker-core = { path = "../core" }
clanker-config = { path = "../config" }
clanker-agent = { path = "../agent", optional = true }

[features]
default = ["agent"]
agent = ["clanker-agent"]
```

### Step 2: Implement Gateway

Create `crates/gateway/src/lib.rs`:

```rust
use axum::{
    extract::{State, WebSocketUpgrade},
    routing::{get, post},
    Router,
};
use clanker_config::Config;
use clanker_core::{types::Message, Result};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

pub struct Gateway {
    config: Arc<RwLock<Config>>,
    message_tx: broadcast::Sender<Message>,
}

impl Gateway {
    pub fn new(config: Config) -> Self {
        let (message_tx, _) = broadcast::channel(1000);

        Self {
            config: Arc::new(RwLock::new(config)),
            message_tx,
        }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/ws", get(ws_handler))
            .route("/api/v1/message", post(send_message))
            .with_state(Arc::new(GatewayState {
                message_tx: self.message_tx.clone(),
            }))
    }

    pub async fn run(&self, bind_addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(bind_addr).await?;
        info!("Gateway listening on {}", bind_addr);

        axum::serve(listener, self.router()).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct GatewayState {
    message_tx: broadcast::Sender<Message>,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<GatewayState>>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

async fn handle_websocket(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<GatewayState>,
) {
    let mut rx = state.message_tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        let _ = state.message_tx.send(message);
                    }
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn send_message(
    State(state): State<Arc<GatewayState>>,
    Json(msg): Json<Message>,
) -> Json<serde_json::Value> {
    let _ = state.message_tx.send(msg);

    Json(serde_json::json!({
        "status": "sent"
    }))
}

pub use axum::extract::ws::Message as WsMessage;
```

---

## Phase 5: Agent Integration (Day 4-5)

### Step 1: Create Agent Crate

Create `crates/agent/Cargo.toml`:

```toml
[package]
name = "clanker-agent"
version.workspace = true
edition.workspace = true

[dependencies]
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
clanker-core = { path = "../core" }
anyhow = "1.0"
```

### Step 2: Implement Anthropic Client

Create `crates/agent/src/lib.rs`:

```rust
use clanker_core::{traits::Agent, types::{AgentResponse, Message, UsageStats}, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicAgent {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicAgent {
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            client,
            api_key,
            model,
            max_tokens,
        }
    }
}

#[async_trait::async_trait]
impl Agent for AnthropicAgent {
    async fn generate(&self, messages: &[Message]) -> Result<AgentResponse> {
        let anthropic_messages: Vec<AnthropicMessage> = messages
            .iter()
            .map(|msg| AnthropicMessage {
                role: "user".to_string(),
                content: msg.text.clone(),
            })
            .collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            messages: anthropic_messages,
            system: Some("You are a helpful AI assistant.".to_string()),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .header("dangerously-allow-browser", "true")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(clanker_core::ClankerError::Agent {
                provider: "anthropic".to_string(),
                message: error_text,
            });
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let content = anthropic_response
            .content
            .iter()
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(AgentResponse {
            id: anthropic_response.id,
            content,
            model: anthropic_response.model,
            finish_reason: anthropic_response.stop_reason,
            usage: UsageStats {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
            },
        })
    }

    async fn health(&self) -> Result<bool> {
        Ok(true)
    }
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: String,
    usage: AnthropicUsage,
}
```

---

## Phase 6: CLI (Day 5-6)

### Step 1: Create CLI Crate

Create `crates/cli/Cargo.toml`:

```toml
[package]
name = "clanker-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "open-clanker"
path = "src/main.rs"

[dependencies]
clap = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
clanker-core = { path = "../core" }
clanker-config = { path = "../config" }
clanker-gateway = { path = "../gateway" }
anyhow = "1.0"
```

### Step 2: Implement CLI

Create `crates/cli/src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "open-clanker")]
#[command(about = "Lightweight AI assistant gateway", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gateway server
    Gateway {
        /// Host to bind to
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,

        /// Port to bind to
        #[arg(short, long, default_value = "18789")]
        port: u16,
    },

    /// Generate default configuration
    ConfigGenerate {
        /// Output path
        #[arg(short, long, default_value = "config.toml")]
        path: String,
    },

    /// Validate configuration
    ConfigValidate,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Gateway { host, port } => {
            let config_path = cli.config.unwrap_or_else(|| "config.toml".to_string());
            let mut config = clanker_config::Config::load_from_path(&config_path)?;
            config.load_env();

            let gateway = clanker_gateway::Gateway::new(config);
            gateway.run(&format!("{}:{}", host, port)).await?;
        }
        Commands::ConfigGenerate { path } => {
            let default_config = clanker_config::generate_default_config();
            std::fs::write(&path, default_config)?;
            println!("Default configuration written to {}", path);
        }
        Commands::ConfigValidate => {
            let config_path = cli.config.unwrap_or_else(|| "config.toml".to_string());
            let _config = clanker_config::Config::load_from_path(&config_path)?;
            println!("Configuration is valid");
        }
    }

    Ok(())
}
```

---

## Phase 7: Docker Setup (Day 6-7)

### Step 1: Create Dockerfile

Create `Dockerfile`:

```dockerfile
# Stage 1: Builder
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig git

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin open-clanker

# Stage 2: Runtime
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates sqlite-libs

# Copy binary from builder
COPY --from=builder /app/target/release/open-clanker /usr/local/bin/open-clanker

# Create necessary directories
RUN mkdir -p /etc/open-clanker /var/lib/open-clanker

# Set permissions
RUN chmod +x /usr/local/bin/open-clanker

# Expose ports
EXPOSE 18789

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:18789/health || exit 1

# Set environment
ENV RUST_LOG=info

# Run the gateway
CMD ["open-clanker", "gateway"]
```

### Step 2: Create Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  open-clanker:
    build: .
    container_name: open-clanker
    restart: unless-stopped
    ports:
      - "18789:18789"
    volumes:
      - ./config.toml:/etc/open-clanker/config.toml:ro
      - ./data:/var/lib/open-clanker
    environment:
      - RUST_LOG=${RUST_LOG:-info}
      - OPENCLAW_TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
      - OPENCLAW_DISCORD_BOT_TOKEN=${DISCORD_BOT_TOKEN}
      - OPENCLAW_ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    healthcheck:
      test: ["CMD", "wget", "--spider", "http://localhost:18789/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s
```

### Step 3: Create .env.example

Create `.env.example`:

```bash
# Server configuration
RUST_LOG=info

# API Keys
ANTHROPIC_API_KEY=sk-ant-your-key-here
OPENAI_API_KEY=sk-openai-your-key-here

# Channel tokens
TELEGRAM_BOT_TOKEN=your-telegram-bot-token
DISCORD_BOT_TOKEN=your-discord-bot-token
```

---

## Phase 8: Testing (Day 7)

### Step 1: Create Basic Tests

Create `crates/core/src/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            ChannelType::Telegram,
            "12345".to_string(),
            "user".to_string(),
            "Hello".to_string(),
        );

        assert_eq!(msg.channel_type, ChannelType::Telegram);
        assert_eq!(msg.channel_id, "12345");
        assert_eq!(msg.sender, "user");
        assert_eq!(msg.text, "Hello");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new(
            ChannelType::Discord,
            "67890".to_string(),
            "user2".to_string(),
            "World".to_string(),
        );

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.text, deserialized.text);
    }
}
```

---

## Phase 9: Documentation (Day 8)

### Step 1: Create README

Create `README_RUST.md`:

```markdown
# Open Clanker

Lightweight, Linux-optimized AI assistant gateway built in Rust.

## Features

- 🚀 **Fast**: Built with Rust for maximum performance
- 🐳 **Docker-ready**: Easy deployment with Docker Compose
- 💬 **Multi-channel**: Telegram and Discord support
- 🤖 **AI-powered**: Anthropic and OpenAI integration
- 📊 **Lightweight**: Minimal resource footprint
- 🔒 **Secure**: Type-safe with memory safety guarantees

## Quick Start

### 1. Generate Configuration

```bash
open-clanker config-generate
```

### 2. Edit `config.toml`

Add your API keys and bot tokens.

### 3. Run with Docker

```bash
cp .env.example .env
# Edit .env with your credentials

docker-compose up -d
```

### 4. Check Status

```bash
curl http://localhost:18789/health
```

## Development

```bash
# Build
cargo build --release

# Run tests
cargo test --all

# Run gateway
cargo run --bin open-clanker -- gateway
```

## License

MIT
```

---

## Next Steps After Day 8

1. **Add Channel Implementations** (Telegram, Discord)
2. **Implement Storage Layer** (SQLite)
3. **Add More Tests** (unit and integration)
4. **Setup CI/CD** (GitHub Actions)
5. **Performance Profiling** (cargo flamegraph)
6. **Security Hardening** (rate limiting, input validation)
7. **Monitoring** (Prometheus metrics, structured logging)
8. **Documentation** (API docs, deployment guide)

---

## Commands Reference

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test --all

# Run with logging
RUST_LOG=debug cargo run -- gateway
```

### Docker

```bash
# Build image
docker build -t open-clanker:latest .

# Run container
docker run -p 18789:18789 -e OPENCLAW_ANTHROPIC_API_KEY=sk-xxx open-clanker:latest

# With docker-compose
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

### Configuration

```bash
# Generate default config
open-clanker config-generate

# Validate config
open-clanker config-validate

# Run with specific config
open-clanker -c /path/to/config.toml gateway
```

---

## Troubleshooting

### Build Errors

If you encounter build errors:

```bash
# Update dependencies
cargo update

# Clean build
cargo clean && cargo build --release
```

### Docker Issues

If Docker build fails:

```bash
# Build with no cache
docker build --no-cache -t open-clanker:latest .

# Check Docker version
docker --version  # Should be 20.10+
```

### Runtime Issues

If gateway won't start:

```bash
# Check logs
RUST_LOG=debug open-clanker gateway

# Validate configuration
open-clanker config-validate

# Check port availability
netstat -tlnp | grep 18789
```

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)
- [Axum Guide](https://docs.rs/axum/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
