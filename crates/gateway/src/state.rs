use crate::broadcast::MessageBroadcaster;
use crate::types::{ConnectionId, ConnectionState};
use clanker_config::Config;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use uuid::Uuid;

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Inner state protected by Arc
    inner: Arc<AppStateInner>,
}

impl AppState {
    /// Create new application state
    pub fn new(config: Config, shutdown_token: CancellationToken) -> Self {
        let inner = Arc::new(AppStateInner::new(config, shutdown_token));

        info!("Application state created");

        Self { inner }
    }

    /// Get broadcaster
    pub fn broadcaster(&self) -> &MessageBroadcaster {
        &self.inner.broadcaster
    }

    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// Get shutdown token
    pub fn shutdown_token(&self) -> &CancellationToken {
        &self.inner.shutdown_token
    }

    /// Add connection
    pub async fn add_connection(&self, id: ConnectionId, state: ConnectionState) {
        let mut connections = self.inner.connections.write().await;
        connections.insert(id, state);

        debug!("Connection {} added. Total connections: {}", id, connections.len());
    }

    /// Remove connection
    pub async fn remove_connection(&self, id: &ConnectionId) {
        let mut connections = self.inner.connections.write().await;
        connections.remove(id);

        debug!("Connection {} removed. Total connections: {}", id, connections.len());
    }

    /// Get connection state
    pub async fn get_connection(&self, id: &ConnectionId) -> Option<ConnectionState> {
        let connections = self.inner.connections.read().await;
        connections.get(id).cloned()
    }

    /// Get all connections
    pub async fn get_all_connections(&self) -> Vec<(ConnectionId, ConnectionState)> {
        let connections = self.inner.connections.read().await;
        connections.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.inner.connections.read().await;
        connections.len()
    }

    /// Increment message count
    pub fn increment_message_count(&self) {
        self.inner.total_messages.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total message count
    pub fn total_message_count(&self) -> u64 {
        self.inner.total_messages.load(Ordering::Relaxed)
    }

    /// Get server start time
    pub fn start_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.inner.start_time
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> i64 {
        (chrono::Utc::now() - self.inner.start_time).num_seconds()
    }

    /// Get uptime as formatted string
    pub fn uptime_formatted(&self) -> String {
        let duration = chrono::Utc::now() - self.inner.start_time;
        let seconds = duration.num_seconds();

        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        format!("{}h {}m {}s", hours, minutes, secs)
    }

    /// Get server version
    pub fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get server identifier
    pub fn server_id(&self) -> Uuid {
        self.inner.server_id
    }
}

/// Inner application state
#[derive(Debug)]
struct AppStateInner {
    /// Message broadcaster
    broadcaster: MessageBroadcaster,
    /// Application configuration
    config: Config,
    /// Active connections (connection_id -> connection_state)
    connections: RwLock<HashMap<ConnectionId, ConnectionState>>,
    /// Total messages processed
    total_messages: AtomicU64,
    /// Server start time
    start_time: chrono::DateTime<chrono::Utc>,
    /// Shutdown token
    shutdown_token: CancellationToken,
    /// Unique server ID
    server_id: Uuid,
}

impl AppStateInner {
    /// Create new inner state
    fn new(config: Config, shutdown_token: CancellationToken) -> Self {
        Self {
            broadcaster: MessageBroadcaster::new(shutdown_token.clone()),
            config,
            connections: RwLock::new(HashMap::new()),
            total_messages: AtomicU64::new(0),
            start_time: chrono::Utc::now(),
            shutdown_token,
            server_id: Uuid::new_v4(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        toml::from_str(include_str!("../../../config-examples/config.toml")).unwrap()
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let state = AppState::new(config, shutdown_token);

        assert_eq!(state.total_message_count(), 0);
        assert!(state.uptime_seconds() >= 0);
        assert_eq!(state.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_management() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let state = AppState::new(config, shutdown_token);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let addr1: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8081".parse().unwrap();

        let conn1 = ConnectionState::new(addr1);
        let conn2 = ConnectionState::new(addr2);

        state.add_connection(id1, conn1.clone()).await;
        state.add_connection(id2, conn2.clone()).await;

        assert_eq!(state.connection_count().await, 2);

        let retrieved = state.get_connection(&id1).await.unwrap();
        assert_eq!(retrieved.id, conn1.id);
        assert_eq!(retrieved.addr, conn1.addr);

        state.remove_connection(&id1).await;

        assert_eq!(state.connection_count().await, 1);
        assert!(state.get_connection(&id1).await.is_none());
        assert!(state.get_connection(&id2).await.is_some());
    }

    #[tokio::test]
    async fn test_message_counting() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let state = AppState::new(config, shutdown_token);

        assert_eq!(state.total_message_count(), 0);

        state.increment_message_count();
        state.increment_message_count();
        state.increment_message_count();

        assert_eq!(state.total_message_count(), 3);
    }

    #[tokio::test]
    async fn test_uptime_tracking() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let state = AppState::new(config, shutdown_token);

        let uptime = state.uptime_seconds();
        assert!(uptime >= 0);
        assert!(uptime < 1); // Should be < 1 second

        let formatted = state.uptime_formatted();
        assert!(formatted.contains("h") || formatted.contains("m") || formatted.contains("s"));
    }

    #[tokio::test]
    async fn test_get_all_connections() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let state = AppState::new(config, shutdown_token);

        let id1 = Uuid::new_v4();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = ConnectionState::new(addr);

        state.add_connection(id1, conn.clone()).await;

        let all = state.get_all_connections().await;

        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, id1);
        assert_eq!(all[0].1.id, conn.id);
    }

    #[test]
    fn test_server_info() {
        let config = create_test_config();
        let shutdown_token = CancellationToken::new();

        let state = AppState::new(config, shutdown_token);

        assert_eq!(state.version(), env!("CARGO_PKG_VERSION"));
        assert!(!state.server_id().is_nil());

        let server_id = state.server_id();
        let server_id2 = state.server_id();
        assert_eq!(server_id, server_id2);
    }
}
