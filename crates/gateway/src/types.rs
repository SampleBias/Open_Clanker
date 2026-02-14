use clanker_core::{ChannelType, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use uuid::Uuid;

/// Unique connection identifier
pub type ConnectionId = Uuid;

/// WebSocket message from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Subscribe to channel updates
    Subscribe {
        channel_id: String,
        channel_type: ChannelType,
    },
    /// Unsubscribe from channel updates
    Unsubscribe {
        channel_id: String,
    },
    /// Send message to channel
    SendMessage {
        channel_id: String,
        channel_type: ChannelType,
        message: String,
    },
    /// Ping to keep connection alive
    Ping { timestamp: u64 },
}

/// WebSocket message to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// Message received from channel
    MessageReceived(Message),
    /// Subscription confirmation
    Subscribed {
        channel_id: String,
        connection_id: ConnectionId,
    },
    /// Unsubscription confirmation
    Unsubscribed {
        channel_id: String,
    },
    /// Response to sent message (includes AI content when available)
    SendResponse {
        success: bool,
        message_id: Option<String>,
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },
    /// Health check response
    Health {
        status: String,
        uptime_seconds: u64,
    },
    /// Pong response
    Pong { timestamp: u64 },
    /// Error message
    Error {
        code: String,
        message: String,
    },
}

impl WsServerMessage {
    /// Create error message
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create send response
    pub fn send_response(
        success: bool,
        message_id: Option<String>,
        error: Option<String>,
        content: Option<String>,
    ) -> Self {
        Self::SendResponse {
            success,
            message_id,
            error,
            content,
        }
    }
}

/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct ConnectionState {
    /// Connection ID
    pub id: ConnectionId,
    /// Client address
    pub addr: SocketAddr,
    /// Connected timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// Subscribed channels (channel_id -> channel_type)
    pub subscriptions: HashMap<String, ChannelType>,
}

impl ConnectionState {
    /// Create new connection state
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            id: Uuid::new_v4(),
            addr,
            connected_at: chrono::Utc::now(),
            subscriptions: HashMap::new(),
        }
    }

    /// Add subscription
    pub fn subscribe(&mut self, channel_id: String, channel_type: ChannelType) {
        self.subscriptions.insert(channel_id, channel_type);
    }

    /// Remove subscription
    pub fn unsubscribe(&mut self, channel_id: &str) {
        self.subscriptions.remove(channel_id);
    }

    /// Check if subscribed to channel
    pub fn is_subscribed(&self, channel_id: &str) -> bool {
        self.subscriptions.contains_key(channel_id)
    }

    /// Get subscription count
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> i64 {
        (chrono::Utc::now() - self.connected_at).num_seconds()
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: String,
    /// Server version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Active WebSocket connections
    pub active_connections: usize,
    /// Total messages processed
    pub total_messages: u64,
    /// Active Worker_Clankers spawned by Master_Clanker
    #[serde(default)]
    pub active_workers: usize,
    /// Maximum Worker_Clankers allowed
    #[serde(default)]
    pub max_workers: usize,
    /// Server timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthResponse {
    /// Create new health response
    pub fn new(
        version: String,
        uptime_seconds: u64,
        active_connections: usize,
        total_messages: u64,
        active_workers: usize,
        max_workers: usize,
    ) -> Self {
        Self {
            status: "healthy".to_string(),
            version,
            uptime_seconds,
            active_connections,
            total_messages,
            active_workers,
            max_workers,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// HTTP status code
    #[serde(skip)]
    pub status: axum::http::StatusCode,
}

impl ApiError {
    /// Create new API error
    pub fn new(code: impl Into<String>, message: impl Into<String>, status: axum::http::StatusCode) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            status,
        }
    }

    /// Bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message, axum::http::StatusCode::BAD_REQUEST)
    }

    /// Unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("UNAUTHORIZED", message, axum::http::StatusCode::UNAUTHORIZED)
    }

    /// Not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message, axum::http::StatusCode::NOT_FOUND)
    }

    /// Internal server error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message, axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Convert ApiError to axum response
use axum::response::IntoResponse;

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        let body = axum::Json(self);

        (status, body).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_client_message_serialization() {
        let msg = WsClientMessage::Ping {
            timestamp: 1234567890,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: WsClientMessage = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(deserialized, WsClientMessage::Ping { .. }));
    }

    #[test]
    fn test_ws_server_message_error() {
        let msg = WsServerMessage::error("TEST_CODE", "Test error message");

        let serialized = serde_json::to_string(&msg).unwrap();
        assert!(serialized.contains("error"));
        assert!(serialized.contains("TEST_CODE"));
        assert!(serialized.contains("Test error message"));
    }

    #[test]
    fn test_connection_state() {
        let mut state = ConnectionState::new("127.0.0.1:8080".parse().unwrap());

        assert_eq!(state.subscription_count(), 0);
        assert!(!state.is_subscribed("test-channel"));

        state.subscribe("test-channel".to_string(), ChannelType::Telegram);

        assert_eq!(state.subscription_count(), 1);
        assert!(state.is_subscribed("test-channel"));

        state.unsubscribe("test-channel");

        assert_eq!(state.subscription_count(), 0);
        assert!(!state.is_subscribed("test-channel"));
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse::new("1.0.0".to_string(), 100, 5, 1000, 0, 5);

        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "1.0.0");
        assert_eq!(health.uptime_seconds, 100);
        assert_eq!(health.active_connections, 5);
        assert_eq!(health.total_messages, 1000);
        assert_eq!(health.active_workers, 0);
        assert_eq!(health.max_workers, 5);
    }

    #[test]
    fn test_api_error_creation() {
        let err = ApiError::bad_request("Invalid input");
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "Invalid input");
        assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::not_found("Resource not found");
        let display = format!("{}", err);
        assert!(display.contains("NOT_FOUND"));
        assert!(display.contains("Resource not found"));
    }
}
