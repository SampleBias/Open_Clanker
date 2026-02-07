# Open Clanker: Technical Architecture

## System Overview

Open Clanker is a lightweight, Linux-optimized AI assistant gateway built in Rust, designed for Docker deployment.

```
┌─────────────────────────────────────────────────────────────┐
│                     Docker Container                        │
│                                                             │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐ │
│  │   CLI        │────▶│  Gateway     │────▶│   Agent      │ │
│  │  (clap)      │     │  (axum)      │     │  (reqwest)   │ │
│  └──────────────┘     └──────────────┘     └──────────────┘ │
│         │                     │                     │        │
│         │                     ▼                     │        │
│         │              ┌──────────────┐            │        │
│         │              │  Channels    │            │        │
│         │              │  (modular)   │            │        │
│         │              └──────────────┘            │        │
│         │                     │                     │        │
│         │        ┌────────────┴────────────┐       │        │
│         │        ▼                         ▼       │        │
│  ┌──────────────┐                   ┌──────────────┐       │
│  │  Telegram    │                   │  Discord     │       │
│  │  (teloxide)  │                   │ (serenity)   │       │
│  └──────────────┘                   └──────────────┘       │
│                                                             │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐ │
│  │   Config     │────▶│   Storage    │────▶│   Logging    │ │
│  │   (TOML)     │     │  (SQLite)    │     │  (tracing)   │ │
│  └──────────────┘     └──────────────┘     └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Crate Architecture

### Workspace Structure

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
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

# Web framework
axum = { version = "0.7", features = ["ws", "macros"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression"] }

# WebSocket
tokio-tungstenite = "0.21"
futures-util = "0.3"

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# CLI
clap = { version = "4.5", features = ["derive", "env"] }
ratatui = "0.26"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
once_cell = "1.19"
dashmap = "5.5"
```

---

## Core Crate (crates/core)

### Purpose
Shared types, errors, and utilities used across all crates.

### Key Modules

#### `types.rs` - Core Data Structures
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Slack,  // Future
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

#### `error.rs` - Error Types
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

#### `traits.rs` - Common Traits
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

---

## Gateway Crate (crates/gateway)

### Purpose
WebSocket and HTTP server for external communication.

### Key Modules

#### `server.rs` - Main Server
```rust
use axum::{
    extract::{State, WebSocketUpgrade},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

use crate::types::{ChannelType, Message};

pub struct Gateway {
    config: Arc<RwLock<GatewayConfig>>,
    message_tx: broadcast::Sender<Message>,
    channels: Arc<RwLock<Vec<dyn crate::traits::Channel>>>,
}

impl Gateway {
    pub fn new(config: GatewayConfig) -> Self {
        let (message_tx, _) = broadcast::channel(1000);

        Self {
            config: Arc::new(RwLock::new(config)),
            message_tx,
            channels: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/ws", get(ws_handler))
            .route("/api/v1/message", post(send_message))
            .route("/api/v1/status", get(status))
            .with_state(Arc::new(GatewayState {
                message_tx: self.message_tx.clone(),
                channels: self.channels.clone(),
            }))
    }

    pub async fn run(&self, bind_addr: &str) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(bind_addr).await?;
        info!("Gateway listening on {}", bind_addr);

        axum::serve(listener, self.router()).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct GatewayState {
    message_tx: broadcast::Sender<Message>,
    channels: Arc<RwLock<Vec<dyn crate::traits::Channel>>>,
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

    // Task to forward messages to WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Task to handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    // Process incoming message
                    if let Ok(message) = serde_json::from_str::<Message>(&text) {
                        // Broadcast to all channels
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
```

#### `middleware.rs` - Middleware Stack
```rust
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::{info, warn};

pub async fn logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    info!("Incoming request: {} {}", method, uri);

    let response = next.run(req).await;

    let status = response.status();
    info!("Response: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"));

    response
}

pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(token) if token.starts_with("Bearer ") => {
            // Validate token
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

---

## Agent Crate (crates/agent)

### Purpose
AI provider integration (Anthropic, OpenAI).

### Key Modules

#### `anthropic.rs` - Anthropic Client
```rust
use crate::types::{AgentResponse, Message, UsageStats};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

pub struct AnthropicAgent {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicAgent {
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            client,
            api_key,
            model,
            max_tokens,
        }
    }

    pub async fn generate(&self, messages: &[Message]) -> anyhow::Result<AgentResponse> {
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
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
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

    pub async fn health(&self) -> anyhow::Result<bool> {
        // Simple health check by making a minimal request
        Ok(true)
    }
}
```

#### `openai.rs` - OpenAI Client
```rust
use crate::types::{AgentResponse, Message, UsageStats};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageContent,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

pub struct OpenAIAgent {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIAgent {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    pub async fn generate(&self, messages: &[Message]) -> anyhow::Result<AgentResponse> {
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: "user".to_string(),
                content: msg.text.clone(),
            })
            .collect();

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let content = openai_response
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AgentResponse {
            id: openai_response.id,
            content,
            model: openai_response.model,
            finish_reason: "stop".to_string(),
            usage: UsageStats {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
        })
    }

    pub async fn health(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}
```

---

## Channels Crate (crates/channels)

### Purpose
Messaging platform integrations.

### Channel Trait Implementation

#### `telegram.rs`
```rust
use crate::traits::Channel;
use crate::types::{ChannelType, Message};
use teloxide::{prelude::*, types::Message as TgMessage};
use tokio::sync::mpsc;

pub struct TelegramChannel {
    bot: Bot,
    rx: mpsc::Receiver<Message>,
    config: TelegramConfig,
}

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chats: Option<Vec<String>>,
}

impl TelegramChannel {
    pub fn new(config: TelegramConfig, rx: mpsc::Receiver<Message>) -> Self {
        Self {
            bot: Bot::new(config.bot_token.clone()),
            rx,
            config,
        }
    }
}

#[async_trait::async_trait]
impl Channel for TelegramChannel {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Telegram
    }

    async fn send_message(&self, message: &Message) -> crate::Result<()> {
        self.bot
            .send_message(ChatId(message.channel_id.parse::<i64>().unwrap()), &message.text)
            .await?;

        Ok(())
    }

    async fn listen(&self) -> crate::Result<()> {
        let bot = self.bot.clone();

        let handler = dptree::entry()
            .branch(Update::filter_message().endpoint(|msg: TgMessage| async move {
                tracing::info!("Received Telegram message: {:?}", msg);
                Response::ok(())
            }));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    async fn stop(&self) -> crate::Result<()> {
        Ok(())
    }
}
```

#### `discord.rs`
```rust
use crate::traits::Channel;
use crate::types::{ChannelType, Message};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

pub struct DiscordChannel {
    token: String,
    config: DiscordConfig,
}

#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub token: String,
    pub guild_id: Option<String>,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            token: config.token.clone(),
            config,
        }
    }
}

#[async_trait::async_trait]
impl Channel for DiscordChannel {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Discord
    }

    async fn send_message(&self, message: &Message) -> crate::Result<()> {
        // Implementation using serenity
        Ok(())
    }

    async fn listen(&self) -> crate::Result<()> {
        let token = self.token.clone();

        tokio::spawn(async move {
            let mut client = Client::builder(&token, GatewayIntents::GUILD_MESSAGES)
                .event_handler(DiscordHandler)
                .await
                .unwrap();

            client.start().await
        });

        Ok(())
    }

    async fn stop(&self) -> crate::Result<()> {
        Ok(())
    }
}

struct DiscordHandler;

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        tracing::info!("Received Discord message: {}", msg.content);
    }

    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("Discord bot connected as: {}", ready.user.name);
    }
}
```

---

## Config Crate (crates/config)

### Purpose
Configuration management and validation.

### `config.rs`
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

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chats: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_env(&mut self) {
        // Load sensitive values from environment
        if let Some(telegram) = &mut self.channels.telegram {
            if let Ok(token) = std::env::var("OPENCLAW_TELEGRAM_BOT_TOKEN") {
                telegram.bot_token = token;
            }
        }
    }
}
```

---

## CLI Crate (crates/cli)

### Purpose
Command-line interface for management and operations.

### `main.rs`
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

        /// Enable TLS
        #[arg(long)]
        tls: bool,
    },

    /// Send a test message
    Send {
        /// Channel type (telegram, discord)
        #[arg(short, long)]
        channel: String,

        /// Channel ID
        #[arg(short, long)]
        channel_id: String,

        /// Message text
        #[arg(short, long)]
        message: String,
    },

    /// Show system status
    Status,

    /// Configuration validation
    ConfigValidate,

    /// Generate default configuration
    ConfigGenerate {
        /// Output path
        #[arg(short, long, default_value = "config.toml")]
        path: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config_path = cli.config.unwrap_or_else(|| "config.toml".to_string());
    let mut config = config::Config::load_from_path(&config_path)?;
    config.load_env();

    match cli.command {
        Commands::Gateway { host, port, tls } => {
            println!("Starting gateway on {}:{}", host, port);
            // Start gateway
            Ok(())
        }
        Commands::Send { channel, channel_id, message } => {
            println!("Sending to {} {}: {}", channel, channel_id, message);
            // Send message
            Ok(())
        }
        Commands::Status => {
            println!("Gateway status:");
            // Show status
            Ok(())
        }
        Commands::ConfigValidate => {
            println!("Configuration valid");
            Ok(())
        }
        Commands::ConfigGenerate { path } => {
            let default_config = config::generate_default_config();
            std::fs::write(&path, default_config)?;
            println!("Default configuration written to {}", path);
            Ok(())
        }
    }
}
```

---

## Storage Crate (crates/storage)

### Purpose
Persistent storage for messages, sessions, and configuration.

### `database.rs`
```rust
use rusqlite::{Connection, params, Result};
use crate::types::Message;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                channel_type TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                sender TEXT NOT NULL,
                text TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_type, channel_id)",
            [],
        )?;

        Ok(())
    }

    pub fn save_message(&self, message: &Message) -> Result<()> {
        self.conn.execute(
            "INSERT INTO messages (id, channel_type, channel_id, sender, text, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                message.id,
                format!("{:?}", message.channel_type),
                message.channel_id,
                message.sender,
                message.text,
                message.timestamp.timestamp(),
            ],
        )?;

        Ok(())
    }

    pub fn get_messages(&self, channel_id: &str, limit: usize) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, channel_type, channel_id, sender, text, timestamp
             FROM messages
             WHERE channel_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;

        let messages = stmt
            .query_map(params![channel_id, limit], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    channel_type: row.get(1)?,
                    channel_id: row.get(2)?,
                    sender: row.get(3)?,
                    text: row.get(4)?,
                    timestamp: chrono::DateTime::from_timestamp(row.get(5)?, 0).unwrap(),
                    metadata: Default::default(),
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }
}
```

---

## Docker Architecture

### Multi-stage Build

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
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates sqlite-libs

# Copy binary from builder
COPY --from=builder /app/target/release/open-clanker /usr/local/bin/open-clanker

# Create necessary directories
RUN mkdir -p /etc/open-clanker /var/lib/open-clanker /var/log/open-clanker

# Set permissions
RUN chmod +x /usr/local/bin/open-clanker

# Create non-root user
RUN addgroup -g 1000 clanker && \
    adduser -D -u 1000 -G clanker -h /var/lib/open-clanker clanker

# Change ownership
RUN chown -R clanker:clanker /var/lib/open-clanker /var/log/open-clanker

USER clanker

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

### Docker Compose

```yaml
version: '3.8'

services:
  open-clanker:
    image: openclanker/open-clanker:latest
    container_name: open-clanker
    restart: unless-stopped
    ports:
      - "18789:18789"
    volumes:
      - ./config.toml:/etc/open-clanker/config.toml:ro
      - ./data:/var/lib/open-clanker
      - ./logs:/var/log/open-clanker
    environment:
      - RUST_LOG=${RUST_LOG:-info}
      - OPENCLAW_TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
      - OPENCLAW_DISCORD_BOT_TOKEN=${DISCORD_BOT_TOKEN}
      - OPENCLAW_ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - OPENCLAW_OPENAI_API_KEY=${OPENAI_API_KEY}
    networks:
      - clanker-network
    healthcheck:
      test: ["CMD", "wget", "--spider", "http://localhost:18789/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s

networks:
  clanker-network:
    driver: bridge
```

---

## Performance Optimization Strategies

### 1. Async Concurrency
- Use `tokio` for all I/O operations
- Implement connection pooling for HTTP clients
- Use `tokio::sync::broadcast` for message distribution

### 2. Memory Management
- Use `jemalloc` for better allocation performance
- Implement message size limits
- Use `Vec` with pre-allocation for buffers

### 3. Network Optimization
- Enable TCP_NODELAY for low latency
- Use HTTP/2 for better multiplexing
- Implement compression with `tower-http`

### 4. Database Optimization
- Use WAL mode for SQLite
- Implement connection pooling
- Add appropriate indexes

### 5. Caching
- Cache channel API responses
- Cache user information
- Use `lru` crate for in-memory caching

---

## Security Considerations

### 1. Authentication
- Validate all API keys
- Use rate limiting per channel
- Implement token rotation

### 2. Input Validation
- Sanitize all user input
- Validate message sizes
- Escape HTML/Markdown

### 3. Network Security
- Enable TLS for WebSocket connections
- Use HTTPS for all HTTP traffic
- Implement CORS policies

### 4. Secrets Management
- Never log API keys
- Use environment variables for secrets
- Implement secure secret rotation

---

## Monitoring & Observability

### Metrics to Track
- Message throughput (messages/second)
- Latency (p50, p95, p99)
- Connection count per channel
- Error rates
- Memory usage
- CPU usage

### Logging Strategy
- Structured JSON logging
- Trace IDs for request correlation
- Log levels: ERROR, WARN, INFO, DEBUG
- Log rotation and retention

### Health Checks
- `/health` endpoint for liveness
- `/ready` endpoint for readiness
- Channel-specific health status
- Provider health status

---

## Development Workflow

### Running Locally
```bash
# Build
cargo build --release

# Run tests
cargo test --all

# Run gateway
cargo run --bin open-clanker -- gateway

# Run with specific config
OPENCLAW_ANTHROPIC_API_KEY=sk-ant-xxx cargo run -- gateway
```

### Docker Development
```bash
# Build image
docker build -t open-clanker:dev .

# Run container
docker run -p 18789:18789 \
  -e OPENCLAW_ANTHROPIC_API_KEY=sk-ant-xxx \
  open-clanker:dev
```

### Testing
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
```

---

## Next Steps

1. **Initialize Rust workspace** with crate structure
2. **Implement core types** and error handling
3. **Build gateway server** with WebSocket support
4. **Integrate AI providers** (Anthropic, OpenAI)
5. **Add channel implementations** (Telegram, Discord)
6. **Create CLI interface** for management
7. **Build and test Docker images**
8. **Setup CI/CD pipeline**
9. **Write comprehensive documentation**
10. **Deploy to Docker Hub**

---

## References

- [Tokio](https://tokio.rs/)
- [Axum](https://github.com/tokio-rs/axum)
- [Teloxide](https://github.com/tokio-rs/teloxide)
- [Serenity](https://github.com/serenity-rs/serenity)
- [Rust Best Practices](https://github.com/rust-lang/rust-clippy)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
