//! Clanker Channel Crate
//!
//! Provides implementations for messaging channels:
//! - Telegram bot (teloxide)
//! - Discord bot (serenity)
//!
//! # Example
//!
//! ```no_run
//! use clanker_channels::{ChannelFactory, Channel};
//! use clanker_core::{ChannelType, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let telegram = ChannelFactory::create_telegram("bot-token".to_string())?;
//!
//!     let message = Message::new(
//!         ChannelType::Telegram,
//!         "12345".to_string(),
//!         "user".to_string(),
//!         "Hello!".to_string(),
//!     );
//!
//!     telegram.send(message).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod telegram;
#[cfg(feature = "discord")]
pub mod discord;

use clanker_core::ChannelType;
use std::sync::Arc;

// Re-exports
pub use error::{ChannelError, Result};

/// Channel trait for all messaging platforms
#[async_trait::async_trait]
pub trait Channel: Send + Sync {
    /// Send a message through the channel
    async fn send(&self, message: clanker_core::Message) -> Result<()>;

    /// Listen for incoming messages (blocking)
    async fn listen(&self) -> Result<()>;

    /// Listen for incoming messages and forward to sender (for gateway integration).
    /// Does not echo; messages are sent to `tx` for processing.
    async fn listen_with_tx(
        &self,
        tx: tokio::sync::mpsc::Sender<clanker_core::Message>,
    ) -> Result<()>;

    /// Get the channel type
    fn channel_type(&self) -> ChannelType;

    /// Check if the channel is connected
    fn is_connected(&self) -> bool;
}

/// Channel factory for creating channel instances
pub struct ChannelFactory;

impl ChannelFactory {
    /// Create a Telegram channel instance
    #[cfg(feature = "telegram")]
    pub fn create_telegram(token: String) -> Result<Box<dyn Channel>> {
        telegram::TelegramChannel::new(token)
            .map(|ch| Box::new(ch) as Box<dyn Channel>)
    }

    /// Create an Arc-wrapped Telegram channel (for shared ownership in gateway)
    #[cfg(feature = "telegram")]
    pub fn create_arc_telegram(token: String) -> Result<Arc<dyn Channel + Send + Sync>> {
        let ch = telegram::TelegramChannel::new(token)?;
        Ok(Arc::new(ch) as Arc<dyn Channel + Send + Sync>)
    }

    /// Create a Discord channel instance
    #[cfg(feature = "discord")]
    pub fn create_discord(token: String) -> Result<Box<dyn Channel>> {
        discord::DiscordChannel::new(token)
            .map(|ch| Box::new(ch) as Box<dyn Channel>)
    }

    /// Create an Arc-wrapped Discord channel (for shared ownership in gateway)
    #[cfg(feature = "discord")]
    pub fn create_arc_discord(token: String) -> Result<Arc<dyn Channel + Send + Sync>> {
        let ch = discord::DiscordChannel::new(token)?;
        Ok(Arc::new(ch) as Arc<dyn Channel + Send + Sync>)
    }

    /// Create a channel based on channel type
    pub fn create(channel_type: ChannelType, token: String) -> Result<Box<dyn Channel>> {
        match channel_type {
            #[cfg(feature = "telegram")]
            ChannelType::Telegram => Self::create_telegram(token),
            #[cfg(feature = "discord")]
            ChannelType::Discord => Self::create_discord(token),
            _ => Err(ChannelError::UnsupportedChannel(format!(
                "Channel type {:?} is not supported",
                channel_type
            ))),
        }
    }

    /// Get supported channel types
    pub fn supported_channels() -> Vec<&'static str> {
        let mut channels = Vec::new();
        #[cfg(feature = "telegram")]
        channels.push("telegram");
        #[cfg(feature = "discord")]
        channels.push("discord");
        channels
    }
}
