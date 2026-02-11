use crate::{Channel, Result};
use crate::error::ChannelError;
use async_trait::async_trait;
use clanker_core::{ChannelType, Message};
use std::sync::atomic::{AtomicBool, Ordering};
use teloxide::{
    prelude::*,
    types::ChatId,
    Bot,
};
use tracing::{debug, info};

/// Telegram channel implementation
pub struct TelegramChannel {
    bot: Bot,
    connected: AtomicBool,
}

impl TelegramChannel {
    /// Create a new Telegram channel
    pub fn new(token: String) -> Result<Self> {
        debug!("Creating Telegram channel");

        let bot = Bot::new(token);

        Ok(Self {
            bot,
            connected: AtomicBool::new(false),
        })
    }

    /// Convert clanker Message to Telegram message
    fn message_to_telegram(msg: &Message) -> Result<(ChatId, String)> {
        let chat_id: i64 = msg.channel_id.parse()
            .map_err(|_| ChannelError::InvalidConfig(format!(
                "Invalid chat ID: {}",
                msg.channel_id
            )))?;

        Ok((ChatId(chat_id), msg.text.clone()))
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    async fn send(&self, message: Message) -> Result<()> {
        debug!("Sending message to Telegram: {}", message.id);

        if !self.is_connected() {
            return Err(ChannelError::ConnectionError(
                "Telegram bot is not connected".to_string(),
            ));
        }

        let (chat_id, text) = Self::message_to_telegram(&message)?;

        self.bot
            .send_message(chat_id, text)
            .await
            .map_err(|e| ChannelError::SendFailed(e.to_string()))?;

        debug!("Message sent successfully");
        Ok(())
    }

    async fn listen(&self) -> Result<()> {
        info!("Starting Telegram listener (legacy echo mode)");
        self.connected.store(true, Ordering::SeqCst);

        let bot = self.bot.clone();
        teloxide::repl(bot, |bot: Bot, msg: teloxide::types::Message| async move {
            let text = msg.text().unwrap_or("");
            bot.send_message(msg.chat.id, format!("Echo: {}", text)).await?;
            Ok(())
        })
        .await;

        Ok(())
    }

    async fn listen_with_tx(
        &self,
        tx: tokio::sync::mpsc::Sender<Message>,
    ) -> Result<()> {
        info!("Starting Telegram listener (forwarding to gateway)");
        self.connected.store(true, Ordering::SeqCst);

        let bot = self.bot.clone();
        let handler = move |_bot: Bot, msg: teloxide::types::Message| {
            let tx = tx.clone();
            async move {
                let text = msg.text().unwrap_or_default();
                if text.is_empty() {
                    return Ok(());
                }
                let sender = msg
                    .from()
                    .map(|u| u.id.0.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let core_msg = Message::new(
                    ChannelType::Telegram,
                    msg.chat.id.0.to_string(),
                    sender,
                    text.to_string(),
                );
                let _ = tx.send(core_msg).await;
                Ok(())
            }
        };

        teloxide::repl(bot, handler).await;

        Ok(())
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Telegram
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_channel_creation() {
        let channel = TelegramChannel::new("test-token".to_string());
        assert!(channel.is_ok());
        assert_eq!(channel.unwrap().channel_type(), ChannelType::Telegram);
    }

    #[test]
    fn test_telegram_channel_type() {
        let channel = TelegramChannel::new("test-token".to_string()).unwrap();
        assert_eq!(channel.channel_type(), ChannelType::Telegram);
    }

    #[test]
    fn test_message_to_telegram() {
        let msg = Message::new(
            ChannelType::Telegram,
            "123456".to_string(),
            "user".to_string(),
            "Hello".to_string(),
        );

        let result = TelegramChannel::message_to_telegram(&msg);
        assert!(result.is_ok());

        let (chat_id, text) = result.unwrap();
        assert_eq!(chat_id, ChatId(123456));
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_message_to_telegram_invalid_chat_id() {
        let msg = Message::new(
            ChannelType::Telegram,
            "invalid".to_string(),
            "user".to_string(),
            "Hello".to_string(),
        );

        let result = TelegramChannel::message_to_telegram(&msg);
        assert!(result.is_err());
    }
}
