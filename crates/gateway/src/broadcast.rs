use crate::types::{WsServerMessage, ConnectionId};
use clanker_core::{ChannelType, Message};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Gateway-wide message broadcaster
#[derive(Debug, Clone)]
pub struct MessageBroadcaster {
    /// Broadcast channel for messages
    tx: broadcast::Sender<WsServerMessage>,
    /// Token for graceful shutdown
    shutdown_token: CancellationToken,
}

impl MessageBroadcaster {
    /// Create new message broadcaster
    pub fn new(shutdown_token: CancellationToken) -> Self {
        let (tx, _rx) = broadcast::channel(1000);

        Self {
            tx,
            shutdown_token,
        }
    }

    /// Subscribe to message broadcasts
    pub fn subscribe(&self) -> broadcast::Receiver<WsServerMessage> {
        self.tx.subscribe()
    }

    /// Broadcast message to all subscribers
    pub async fn broadcast(&self, message: WsServerMessage) -> Result<(), broadcast::error::SendError<WsServerMessage>> {
        let _ = self.tx.send(message);
        Ok(())
    }

    /// Send message to specific channel subscribers
    pub async fn send_to_channel(
        &self,
        message: &Message,
    ) -> Result<(), broadcast::error::SendError<WsServerMessage>> {
        let ws_message = WsServerMessage::MessageReceived(message.clone());

        debug!(
            "Broadcasting message to channel {}",
            message.channel_id
        );

        self.broadcast(ws_message).await
    }

    /// Send error to all subscribers
    pub async fn send_error(
        &self,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<(), broadcast::error::SendError<WsServerMessage>> {
        let code_str = code.into();
        let msg_str = message.into();
        
        let ws_message = WsServerMessage::error(&code_str, &msg_str);

        warn!("Broadcasting error: {} - {}", code_str, msg_str);

        self.broadcast(ws_message).await
    }

    /// Check if broadcaster is still active
    pub fn is_active(&self) -> bool {
        !self.shutdown_token.is_cancelled()
    }

    /// Get number of receivers (active connections)
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// Message sender for specific connection
#[derive(Debug)]
pub struct ConnectionSender {
    /// Connection ID
    id: ConnectionId,
    /// Broadcast receiver
    rx: broadcast::Receiver<WsServerMessage>,
}

impl ConnectionSender {
    /// Create new connection sender
    pub fn new(id: ConnectionId, rx: broadcast::Receiver<WsServerMessage>) -> Self {
        Self { id, rx }
    }

    /// Receive next message
    pub async fn recv(&mut self) -> Result<WsServerMessage, broadcast::error::RecvError> {
        self.rx.recv().await
    }

    /// Try to receive next message without blocking
    pub fn try_recv(&mut self) -> Result<WsServerMessage, broadcast::error::TryRecvError> {
        self.rx.try_recv()
    }

    /// Get connection ID
    pub fn id(&self) -> ConnectionId {
        self.id
    }
}

/// Message filter for channel subscriptions
#[derive(Debug, Clone)]
pub struct MessageFilter {
    /// Channel ID to filter for
    channel_id: Option<String>,
    /// Channel type to filter for
    channel_type: Option<ChannelType>,
}

impl MessageFilter {
    /// Create new message filter
    pub fn new(channel_id: Option<String>, channel_type: Option<ChannelType>) -> Self {
        Self {
            channel_id,
            channel_type,
        }
    }

    /// Check if message matches filter
    pub fn matches(&self, message: &WsServerMessage) -> bool {
        match message {
            WsServerMessage::MessageReceived(msg) => {
                // Check channel ID filter
                if let Some(ref channel_id) = self.channel_id {
                    if &msg.channel_id != channel_id {
                        return false;
                    }
                }

                // Check channel type filter
                if let Some(channel_type) = self.channel_type {
                    if msg.channel_type != channel_type {
                        return false;
                    }
                }

                true
            }
            // Pass through all non-message types
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcaster_creation() {
        let shutdown_token = CancellationToken::new();
        let broadcaster = MessageBroadcaster::new(shutdown_token);

        assert!(broadcaster.is_active());
        assert_eq!(broadcaster.receiver_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcaster_subscribe() {
        let shutdown_token = CancellationToken::new();
        let broadcaster = MessageBroadcaster::new(shutdown_token);

        let _rx = broadcaster.subscribe();
        assert_eq!(broadcaster.receiver_count(), 1);

        let _rx2 = broadcaster.subscribe();
        assert_eq!(broadcaster.receiver_count(), 2);
    }

    #[tokio::test]
    async fn test_broadcaster_broadcast() {
        let shutdown_token = CancellationToken::new();
        let broadcaster = MessageBroadcaster::new(shutdown_token);

        let mut rx = broadcaster.subscribe();

        let message = Message::new(
            ChannelType::Telegram,
            "test-channel".to_string(),
            "user123".to_string(),
            "Hello, world!".to_string(),
        );

        broadcaster.send_to_channel(&message).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert!(matches!(received, WsServerMessage::MessageReceived(_)));
    }

    #[tokio::test]
    async fn test_message_filter() {
        let filter = MessageFilter::new(
            Some("test-channel".to_string()),
            Some(ChannelType::Telegram),
        );

        let matching_msg = Message::new(
            ChannelType::Telegram,
            "test-channel".to_string(),
            "user123".to_string(),
            "Hello".to_string(),
        );

        let non_matching_channel = Message::new(
            ChannelType::Telegram,
            "other-channel".to_string(),
            "user123".to_string(),
            "Hello".to_string(),
        );

        let non_matching_type = Message::new(
            ChannelType::Discord,
            "test-channel".to_string(),
            "user123".to_string(),
            "Hello".to_string(),
        );

        let ws_msg = WsServerMessage::MessageReceived(matching_msg);
        assert!(filter.matches(&ws_msg));

        let ws_msg = WsServerMessage::MessageReceived(non_matching_channel);
        assert!(!filter.matches(&ws_msg));

        let ws_msg = WsServerMessage::MessageReceived(non_matching_type);
        assert!(!filter.matches(&ws_msg));

        // Non-message types should pass through
        let ping_msg = WsServerMessage::Pong { timestamp: 0 };
        assert!(filter.matches(&ping_msg));
    }

    #[tokio::test]
    async fn test_connection_sender() {
        let shutdown_token = CancellationToken::new();
        let broadcaster = MessageBroadcaster::new(shutdown_token);

        let mut rx = ConnectionSender::new(Uuid::new_v4(), broadcaster.subscribe());

        let message = WsServerMessage::Pong { timestamp: 123 };
        broadcaster.broadcast(message).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert!(matches!(received, WsServerMessage::Pong { .. }));
    }
}
