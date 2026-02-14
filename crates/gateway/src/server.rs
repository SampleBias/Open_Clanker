use crate::handlers::{health_check, root, websocket_handler};
use crate::middleware::{cors_layer, security_headers_middleware};
use crate::processor;
use crate::state::AppState;
use axum::{routing::{any, get, Router}};
use clanker_config::Config;
use clanker_core::Message;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Gateway Server
pub struct GatewayServer {
    config: Config,
    state: AppState,
    shutdown_token: CancellationToken,
}

impl GatewayServer {
    pub fn new(config: Config, shutdown_token: CancellationToken) -> Self {
        let state = AppState::new(config.clone(), shutdown_token.clone());

        info!("Gateway server created");
        info!("  Host: {}", config.server.host);
        info!("  Port: {}", config.server.port);
        info!("  Provider: {}", config.agent.provider);
        info!("  Model: {}", config.agent.model);

        Self {
            config,
            state,
            shutdown_token,
        }
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("Gateway server listening on {}", addr);
        info!("  WebSocket: ws://{}/ws", addr);
        info!("  Health: http://{}/health", addr);
        info!("  API: http://{}/", addr);

        let app = self.build_router();
        self.setup_graceful_shutdown();

        // Spawn channel listeners and processing loop when channels are configured
        let state = self.state.clone();
        if !state.channels().is_empty() {
            let (tx, mut rx) = mpsc::channel::<Message>(256);
            let shutdown = state.shutdown_token().clone();

            // Spawn channel listeners
            for ch in state.channels() {
                let ch = ch.clone();
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = ch.listen_with_tx(tx).await {
                        error!("Channel {} listener error: {}", ch.channel_type(), e);
                    }
                });
            }

            // Spawn processing loop
            let state_clone = state.clone();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(incoming) = rx.recv() => {
                            match processor::process_message(&state_clone, &incoming).await {
                                Ok(response) => {
                                    if let Some(ch) = state.channel_for(incoming.channel_type) {
                                        if let Err(e) = ch.send(response).await {
                                            error!("Failed to send to {}: {}", incoming.channel_type, e);
                                        }
                                    } else {
                                        warn!("No channel for type {:?}", incoming.channel_type);
                                    }
                                }
                                Err(e) => error!("Processor error: {}", e),
                            }
                        }
                        _ = shutdown.cancelled() => break,
                    }
                }
            });
        }

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                self.shutdown_token.cancelled().await;
            })
            .await?;

        info!("Gateway server shutdown complete");
        Ok(())
    }

    fn build_router(&self) -> Router {
        Router::new()
            .route("/", get(root))
            .route("/health", get(health_check))
            .route("/ws", any(websocket_handler))
            .with_state(self.state.clone())
            .layer(cors_layer())
            .route_layer(axum::middleware::from_fn(security_headers_middleware))
    }

    fn setup_graceful_shutdown(&self) {
        let shutdown_token = self.shutdown_token.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
            info!("Ctrl+C received, initiating graceful shutdown...");
            shutdown_token.cancel();
        });

        let shutdown_token = self.shutdown_token.clone();
        tokio::spawn(async move {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to setup SIGTERM handler")
                .recv()
                .await
                .expect("Failed to receive SIGTERM");
            info!("SIGTERM received, initiating graceful shutdown...");
            shutdown_token.cancel();
        });
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.config.server.host, self.config.server.port)
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        let config_str = include_str!("../../../config-examples/config.toml");
        toml::from_str(config_str).unwrap()
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let server = GatewayServer::new(config, shutdown_token);

        assert_eq!(server.address(), "0.0.0.0:18789");
        assert_eq!(server.state().version(), env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_router_creation() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let server = GatewayServer::new(config, shutdown_token);
        let _router = server.build_router();
    }
}
