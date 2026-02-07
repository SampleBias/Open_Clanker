# Open Clanker Migration Plan: TypeScript → Rust

## Executive Summary
Transform OpenClaw into **Open Clanker**: a lightweight, Linux-optimized, Rust-based AI assistant gateway deployable via Docker.

---

## Phase 0: Analysis & Decisions

### Core Components to Keep
✅ **Essential:**
- Gateway server (WebSocket/HTTP)
- Agent system (Anthropic/OpenAI integration)
- Basic CLI interface
- Configuration management
- Core channels: Telegram, Discord (start with 2)

❌ **Remove:**
- All mobile apps (iOS/Android)
- macOS app
- Complex channel ecosystem (start lean)
- Extensive plugin system (keep minimal)
- TUI and Web UI (simplified CLI only)
- Browser automation (complexity/weight)
- Canvas rendering (can add later)
- Cron/scheduling (can add later)
- Multiple auth providers (keep Anthropic + OpenAI)

### Architecture Decisions
- **Backend**: Pure Rust with Tokio async runtime
- **Gateway**: Tokio-tungstenite (WebSocket) + Actix-web/axum (HTTP)
- **Channels**: Official Rust SDKs where available
- **Config**: TOML-based configuration
- **CLI**: clap-rs + ratatui (for simple terminal output)
- **Storage**: SQLite with rusqlite (or sled for performance)
- **Docker**: Single optimized image (alpine-based)

---

## Phase 1: Project Structure Setup (Week 1-2)

### 1.1 Create Rust Workspace
```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "core",
    "gateway",
    "channels",
    "agent",
    "cli",
    "config",
]
resolver = "2"

[workspace.dependencies]
# Shared dependencies
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1"
thiserror = "1"
```

### 1.2 Crate Organization
```
open_clanker/
├── Cargo.toml (workspace)
├── core/           # Shared types, errors, utilities
├── gateway/        # WebSocket/HTTP server
├── channels/       # Channel implementations
├── agent/          # AI agent integration
├── cli/            # Command-line interface
├── config/         # Configuration management
├── Dockerfile
├── docker-compose.yml
└── README.md
```

### 1.3 Initial Dependencies
```toml
# core/Cargo.toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }

# gateway/Cargo.toml
[dependencies]
axum = { version = "0.7", features = ["ws"] }
tokio = { workspace = true }
tokio-tungstenite = "0.21"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# agent/Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { workspace = true }
serde_json = { workspace = true }
```

---

## Phase 2: Core Data Structures (Week 2-3)

### 2.1 Shared Types (core/src/types.rs)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel: ChannelType,
    pub sender: String,
    pub text: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Telegram,
    Discord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub url: String,
    pub mime_type: String,
    pub size: u64,
}
```

### 2.2 Error Handling (core/src/error.rs)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClankerError {
    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 2.3 Configuration Schema (config/src/schema.rs)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub channels: ChannelsConfig,
    pub agent: AgentConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelsConfig {
    pub telegram: Option<TelegramConfig>,
    pub discord: Option<DiscordConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_users: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub guild_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AgentConfig {
    pub provider: AgentProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentProvider {
    Anthropic,
    OpenAI,
}
```

---

## Phase 3: Gateway Server (Week 3-4)

### 3.1 WebSocket Server (gateway/src/ws.rs)
```rust
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use tokio::sync::broadcast;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tx): State<broadcast::Sender<String>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to messages
    let mut rx = tx.subscribe();

    // Spawn task to forward messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from client
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Process incoming message
                    tracing::info!("Received: {}", text);
                }
                Message::Close(_) => break,
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

### 3.2 HTTP API (gateway/src/http.rs)
```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

use crate::types::Message;

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/message", post(send_message))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn send_message(
    State(tx): State<broadcast::Sender<String>>,
    Json(msg): Json<Message>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Process and send message
    let serialized = serde_json::to_string(&msg).unwrap();
    let _ = tx.send(serialized);

    Ok(Json(serde_json::json!({
        "status": "sent",
        "message_id": msg.id
    })))
}
```

### 3.3 Main Gateway (gateway/src/main.rs)
```rust
use axum::Router;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ws;
mod http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, _rx) = broadcast::channel(1000);

    let app = Router::new()
        .nest("/", http::create_router())
        .route("/ws", ws::ws_handler)
        .with_state(tx);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:18789").await?;
    tracing::info!("Gateway listening on 0.0.0.0:18789");

    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Phase 4: Agent Integration (Week 4-5)

### 4.1 Anthropic Client (agent/src/anthropic.rs)
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    pub async fn chat(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages,
            system: None,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let anthropic_response: AnthropicResponse = response.json().await?;

        Ok(anthropic_response
            .content
            .iter()
            .filter_map(|c| c.text.as_ref())
            .collect::<Vec<_>>()
            .join("\n"))
    }
}
```

### 4.2 OpenAI Client (agent/src/openai.rs)
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> anyhow::Result<String> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let openai_response: OpenAIResponse = response.json().await?;

        Ok(openai_response
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}
```

---

## Phase 5: Channel Implementations (Week 5-6)

### 5.1 Telegram Channel (channels/src/telegram.rs)
```rust
use teloxide::{prelude::*, types::Message as TgMessage};
use tokio::sync::mpsc;

pub struct TelegramBot {
    bot: Bot,
    rx: mpsc::Receiver<String>,
}

impl TelegramBot {
    pub fn new(token: String, rx: mpsc::Receiver<String>) -> Self {
        Self {
            bot: Bot::new(token),
            rx,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let handler = dptree::entry()
            .branch(Update::filter_message().endpoint(|msg: TgMessage| async move {
                tracing::info!("Received message: {:?}", msg);
                Response::ok(())
            }));

        Dispatcher::builder(self.bot.clone(), handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> anyhow::Result<()> {
        self.bot
            .send_message(ChatId(chat_id), text)
            .await?;
        Ok(())
    }
}
```

### 5.2 Discord Channel (channels/src/discord.rs)
```rust
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

pub struct DiscordBot {
    token: String,
}

impl DiscordBot {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut client = Client::builder(&self.token, GatewayIntents::GUILD_MESSAGES)
            .event_handler(Handler)
            .await?;

        client.start().await?;
        Ok(())
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        tracing::info!("Received message: {}", msg.content);

        // Process message and send response
    }

    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);
    }
}
```

---

## Phase 6: CLI Interface (Week 6-7)

### 6.1 CLI Commands (cli/src/main.rs)
```rust
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "open-clanker")]
#[command(about = "Lightweight AI assistant gateway", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gateway server
    Gateway {
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,
        #[arg(short, long, default_value = "18789")]
        port: u16,
    },
    /// Send a message
    Send {
        #[arg(short, long)]
        channel: String,
        #[arg(short, long)]
        message: String,
    },
    /// Show status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Gateway { host, port } => {
            println!("Starting gateway on {}:{}", host, port);
            // Start gateway
        }
        Commands::Send { channel, message } => {
            println!("Sending to {}: {}", channel, message);
            // Send message
        }
        Commands::Status => {
            println!("Gateway status:");
            // Show status
        }
    }

    Ok(())
}
```

---

## Phase 7: Docker Deployment (Week 7-8)

### 7.1 Multi-stage Dockerfile
```dockerfile
# Build stage
FROM rust:1.75-alpine as builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY gateway ./gateway
COPY agent ./agent
COPY channels ./channels
COPY cli ./cli
COPY config ./config

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/open-clanker /usr/local/bin/open-clanker

# Create config directory
RUN mkdir -p /etc/open-clanker

# Expose ports
EXPOSE 18789

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
  CMD wget --no-verbose --tries=1 --spider http://localhost:18789/health || exit 1

# Run the gateway
CMD ["open-clanker", "gateway", "--host", "0.0.0.0", "--port", "18789"]
```

### 7.2 Docker Compose
```yaml
version: '3.8'

services:
  clanker:
    image: openclanker/open-clanker:latest
    container_name: open-clanker
    restart: unless-stopped
    ports:
      - "18789:18789"
    volumes:
      - ./config:/etc/open-clanker:ro
      - ./data:/var/lib/open-clanker
    environment:
      - RUST_LOG=info
      - OPENCLAW_TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
      - OPENCLAW_ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - OPENCLAW_OPENAI_API_KEY=${OPENAI_API_KEY}
    healthcheck:
      test: ["CMD", "wget", "--spider", "http://localhost:18789/health"]
      interval: 30s
      timeout: 3s
      retries: 3
```

---

## Phase 8: Optimization & Testing (Week 8-9)

### 8.1 Performance Optimizations
- Use `jemalloc` for memory management
- Implement connection pooling
- Add rate limiting per channel
- Optimize message buffering
- Profile with `cargo flamegraph`

### 8.2 Security Hardening
- Validate all inputs
- Sanitize HTML/markdown
- Rate limit API calls
- Implement proper auth
- Secure WebSocket connections

### 8.3 Monitoring
- Add Prometheus metrics
- Structured logging
- Health check endpoints
- Error tracking

---

## Phase 9: Documentation & Release (Week 9-10)

### 9.1 Documentation
- `README.md` with quick start
- `CONFIG.md` for configuration
- `DEPLOY.md` for Docker deployment
- `API.md` for HTTP/WebSocket API

### 9.2 Docker Hub Setup
1. Create `openclanker` organization
2. Setup automated builds
3. Tag strategy: `latest`, `v1.0.0`, `stable`
4. Multi-arch builds (amd64, arm64)

### 9.3 Release Process
```bash
# Build and test
cargo test --all-features
cargo clippy -- -D warnings

# Build Docker images
docker buildx build --platform linux/amd64,linux/arm64 -t openclanker/open-clanker:latest .
docker buildx build --platform linux/amd64,linux/arm64 -t openclanker/open-clanker:v1.0.0 .

# Push to Docker Hub
docker push openclanker/open-clanker:latest
docker push openclanker/open-clanker:v1.0.0
```

---

## Migration Checklist

### Phase 0: Planning ✅
- [x] Analyze current architecture
- [x] Identify core components
- [x] Define scope for v1.0
- [ ] Create project roadmap

### Phase 1: Setup
- [ ] Initialize Rust workspace
- [ ] Set up CI/CD
- [ ] Create development environment

### Phase 2: Core
- [ ] Implement shared types
- [ ] Implement error handling
- [ ] Implement configuration schema

### Phase 3: Gateway
- [ ] Implement WebSocket server
- [ ] Implement HTTP API
- [ ] Add authentication

### Phase 4: Agent
- [ ] Implement Anthropic client
- [ ] Implement OpenAI client
- [ ] Add message routing

### Phase 5: Channels
- [ ] Implement Telegram bot
- [ ] Implement Discord bot
- [ ] Add channel abstraction

### Phase 6: CLI
- [ ] Implement CLI commands
- [ ] Add configuration CLI
- [ ] Add status reporting

### Phase 7: Docker
- [ ] Create multi-stage Dockerfile
- [ ] Create docker-compose.yml
- [ ] Test container deployment

### Phase 8: Testing
- [ ] Unit tests
- [ ] Integration tests
- [ ] Performance profiling

### Phase 9: Release
- [ ] Documentation
- [ ] Docker Hub setup
- [ ] v1.0 release

---

## Key Decisions & Trade-offs

### What We're Removing
| Component | Reason | When to Add Back |
|-----------|---------|-----------------|
| Mobile apps | High complexity, Linux-focused | Demand from users |
| macOS app | Linux-focused only | N/A |
| Web UI | Adds complexity | v1.1 or later |
| TUI | Keep simple CLI | N/A |
| Browser automation | Security/performance | v2.0 |
| Canvas | Not core to v1 | v1.2 |
| Cron | Feature creep | v1.3 |
| Many channels | Start lean | As requested |

### Why Rust?
- **Performance**: Zero-cost abstractions, no GC
- **Memory Safety**: Prevents entire classes of bugs
- **Concurrency**: Tokio async runtime
- **Linux-optimized**: Smaller binaries, lower resource usage
- **Ecosystem**: Excellent async libraries

### Why Docker?
- **Simplicity**: Single binary, no dependency hell
- **Portability**: Run anywhere with Docker
- **Updates**: Easy rollouts with Docker Hub
- **Isolation**: Security benefits

---

## Estimated Timeline
- **Week 1-2**: Setup and core types
- **Week 3-4**: Gateway implementation
- **Week 4-5**: Agent integration
- **Week 5-6**: Channel implementations
- **Week 6-7**: CLI and configuration
- **Week 7-8**: Docker and deployment
- **Week 8-9**: Testing and optimization
- **Week 9-10**: Documentation and release

**Total: 10 weeks** for a production-ready v1.0

---

## Success Metrics
- Binary size < 20MB (stripped)
- Memory usage < 100MB (idle)
- < 50ms latency for message processing
- 99.9% uptime
- Support 100+ concurrent connections
- Zero runtime security vulnerabilities

---

## Next Steps
1. **Review and approve this plan**
2. **Set up GitHub repository**
3. **Begin Phase 1: Project Structure**
4. **Establish CI/CD pipeline**
5. **Create initial commit with workspace structure**

---

## Resources & References
- [Tokio](https://tokio.rs/)
- [Axum](https://github.com/tokio-rs/axum)
- [Teloxide (Telegram)](https://github.com/tokio-rs/teloxide)
- [Serenity (Discord)](https://github.com/serenity-rs/serenity)
- [Docker best practices](https://docs.docker.com/develop/dev-best-practices/)
- [Rust packaging guidelines](https://doc.rust-lang.org/cargo/guide.html)
