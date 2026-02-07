use crate::{Channel, Result};
use crate::error::ChannelError;
use async_trait::async_trait;
use clanker_core::{ChannelType, Message};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, info};

/// Discord channel implementation
pub struct DiscordChannel {
    token: String,
    connected: AtomicBool,
}

impl DiscordChannel {
    /// Create a new Discord channel
    pub fn new(token: String) -> Result<Self> {
        debug!("Creating Discord channel");

        Ok(Self {
            token,
            connected: AtomicBool::new(false),
        })
    }

    /// Convert clanker Message to Discord message
    fn message_to_discord(msg: &Message) -> Result<(String, String)> {
        let channel_id = msg.channel_id.clone();
        let content = msg.text.clone();
        Ok((channel_id, content))
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    async fn send(&self, message: Message) -> Result<()> {
        debug!("Sending message to Discord: {}", message.id);

        if !self.is_connected() {
            return Err(ChannelError::ConnectionError(
                "Discord bot is not connected".to_string(),
            ));
        }

        let (_channel_id, _content) = Self::message_to_discord(&message)?;

        // TODO: Implement actual Discord sending
        // This requires the serenity client to be initialized
        info!("Discord send not yet fully implemented - placeholder");

        debug!("Message placeholder sent");
        Ok(())
    }

    async fn listen(&self) -> Result<()> {
        info!("Starting Discord listener");

        // Set connected state
        self.connected.store(true, Ordering::SeqCst);

        // TODO: Implement actual Discord listening
        // This requires serenity client initialization
        info!("Discord listen not yet fully implemented - placeholder");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok(())
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Discord
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_channel_creation() {
        let channel = DiscordChannel::new("test-token".to_string());
        assert!(channel.is_ok());
        assert_eq!(channel.unwrap().channel_type(), ChannelType::Discord);
    }

    #[test]
    fn test_discord_channel_type() {
        let channel = DiscordChannel::new("test-token".to_string()).unwrap();
        assert_eq!(channel.channel_type(), ChannelType::Discord);
    }

    #[test]
    fn test_discord_message_conversion() {
        let msg = Message::new(
            ChannelType::Discord,
            "123456789".to_string(),
            "user".to_string(),
            "Hello".to_string(),
        );

        let (channel_id, text) = DiscordChannel::message_to_discord(&msg).unwrap();
        assert_eq!(channel_id, "123456789");
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_discord_message_conversion_complex() {
        let msg = Message::new(
            ChannelType::Discord,
            "987654321".to_string(),
            "user".to_string(),
            "Test message".to_string(),
        );

        let (channel_id, text) = DiscordChannel::message_to_discord(&msg).unwrap();
        assert_eq!(channel_id, "987654321");
        assert_eq!(text, "Test message");
    }
}
