//! Open Clanker Gateway - WebSocket + HTTP Server
//!
//! A production-ready gateway server with:
//! - WebSocket support for real-time messaging
//! - Server-Sent Events (SSE) for streaming
//! - REST API for health checks
//! - Message broadcasting system
//! - Graceful shutdown
//! - CORS, compression, security headers
//!
//! # Example
//!
//! ```no_run
//! use clanker_gateway::GatewayServer;
//! use clanker_config::Config;
//! use tokio_util::sync::CancellationToken;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = Config::load_from_path("config.toml")?;
//!
//!     // Create shutdown token
//!     let shutdown_token = CancellationToken::new();
//!
//!     // Create and start server
//!     let server = GatewayServer::new(config, shutdown_token);
//!     server.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod broadcast;
pub mod handlers;
pub mod middleware;
pub mod processor;
pub mod server;
pub mod state;
pub mod types;

// Re-export commonly used types
pub use server::GatewayServer;
pub use state::AppState;
pub use types::{
    ApiError, ConnectionId, ConnectionState, HealthResponse,
    WsClientMessage, WsServerMessage,
};
