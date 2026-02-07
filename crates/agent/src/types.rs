use async_trait::async_trait;
use clanker_core::ChannelType;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Agent errors
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Rate limited: retry after {0:?}")]
    RateLimited(Option<Duration>),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Agent message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub finish_reason: String,
    pub usage: Usage,
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Stream chunk for streaming responses
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
    pub usage: Option<Usage>,
}

/// System prompt configuration
#[derive(Debug, Clone)]
pub struct SystemPrompt {
    pub content: String,
    pub channel_type: Option<ChannelType>,
}

impl SystemPrompt {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            channel_type: None,
        }
    }

    pub fn with_channel_type(mut self, channel_type: ChannelType) -> Self {
        self.channel_type = Some(channel_type);
        self
    }
}

/// Default system prompts
pub mod system_prompts {
    use super::SystemPrompt;
    use clanker_core::ChannelType;

    pub const DEFAULT: &str = "You are a helpful AI assistant.";

    pub fn for_channel(channel_type: ChannelType) -> SystemPrompt {
        let prompt = match channel_type {
            ChannelType::Telegram => "You are a helpful AI assistant for Telegram. Keep responses concise and engaging.",
            ChannelType::Discord => "You are a helpful AI assistant for Discord. Be conversational and use Discord-friendly formatting.",
            ChannelType::Slack => "You are a helpful AI assistant for Slack. Keep responses professional and clear.",
            ChannelType::WhatsApp => "You are a helpful AI assistant for WhatsApp. Keep responses friendly and concise.",
        };

        SystemPrompt::new(prompt).with_channel_type(channel_type)
    }

    pub fn default() -> SystemPrompt {
        SystemPrompt::new(DEFAULT)
    }
}

/// Agent trait for all providers
#[async_trait]
pub trait Agent: Send + Sync {
    /// Send chat completion request
    async fn chat(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError>;

    /// Send streaming chat completion request
    async fn chat_stream(
        &self,
        messages: Vec<AgentMessage>,
    ) -> Result<
        Box<dyn Stream<Item = Result<StreamChunk, AgentError>> + Send + Unpin>,
        AgentError,
    >;

    /// Get agent provider name
    fn provider(&self) -> &str;

    /// Get model name
    fn model(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_message_serialization() {
        let msg = AgentMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_system_prompt_creation() {
        let prompt = SystemPrompt::new("Test prompt");
        assert_eq!(prompt.content, "Test prompt");
        assert!(prompt.channel_type.is_none());
    }

    #[test]
    fn test_channel_specific_prompts() {
        let telegram_prompt = system_prompts::for_channel(ChannelType::Telegram);
        assert!(telegram_prompt.content.contains("Telegram"));

        let discord_prompt = system_prompts::for_channel(ChannelType::Discord);
        assert!(discord_prompt.content.contains("Discord"));
    }
}
