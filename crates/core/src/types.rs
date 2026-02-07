use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for messages
pub type MessageId = String;

/// User/channel identifier
pub type UserId = String;

/// Message sent through the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: MessageId,
    pub channel_type: ChannelType,
    pub channel_id: String,
    pub sender: UserId,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: MessageMetadata,
}

impl Message {
    /// Create a new message
    pub fn new(
        channel_type: ChannelType,
        channel_id: String,
        sender: String,
        text: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            channel_type,
            channel_id,
            sender,
            text,
            timestamp: Utc::now(),
            metadata: MessageMetadata::default(),
        }
    }

    /// Create a new message with custom timestamp
    pub fn new_with_timestamp(
        channel_type: ChannelType,
        channel_id: String,
        sender: String,
        text: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            channel_type,
            channel_id,
            sender,
            text,
            timestamp,
            metadata: MessageMetadata::default(),
        }
    }

    /// Add an attachment to the message
    pub fn add_attachment(mut self, attachment: Attachment) -> Self {
        self.metadata.attachments.push(attachment);
        self
    }

    /// Set reply-to message
    pub fn set_reply_to(mut self, reply_to: MessageId) -> Self {
        self.metadata.reply_to = Some(reply_to);
        self
    }

    /// Add a mention
    pub fn add_mention(mut self, user_id: UserId) -> Self {
        self.metadata.mentions.push(user_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MessageMetadata {
    pub attachments: Vec<Attachment>,
    pub reply_to: Option<MessageId>,
    pub mentions: Vec<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    pub id: String,
    pub url: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(url: String, mime_type: String, size_bytes: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            mime_type,
            size_bytes,
        }
    }
}

/// Supported channel types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Telegram,
    Discord,
    Slack,
    WhatsApp,
}

impl ChannelType {
    /// Get channel type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelType::Telegram => "telegram",
            ChannelType::Discord => "discord",
            ChannelType::Slack => "slack",
            ChannelType::WhatsApp => "whatsapp",
        }
    }

    /// Parse channel type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "telegram" => Some(ChannelType::Telegram),
            "discord" => Some(ChannelType::Discord),
            _ => None,
        }
    }
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// AI agent response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    pub id: String,
    pub content: String,
    pub model: String,
    pub finish_reason: String,
    pub usage: UsageStats,
}

impl AgentResponse {
    /// Create a new agent response
    pub fn new(
        id: String,
        content: String,
        model: String,
        finish_reason: String,
        usage: UsageStats,
    ) -> Self {
        Self {
            id,
            content,
            model,
            finish_reason,
            usage,
        }
    }
}

/// Usage statistics for AI agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl UsageStats {
    /// Create new usage stats
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            total_tokens: prompt_tokens + completion_tokens,
            prompt_tokens,
            completion_tokens,
        }
    }

    /// Calculate cost based on provider (simplified)
    pub fn calculate_cost(&self, provider: &str, model: &str) -> f64 {
        // Simplified cost calculation
        // Actual costs should be fetched from provider pricing
        let (input_cost, output_cost) = match (provider.to_lowercase().as_str(), model.to_lowercase().as_str()) {
            // Anthropic Claude
            ("anthropic", m) if m.contains("sonnet") => (3.0, 15.0),
            ("anthropic", m) if m.contains("opus") => (15.0, 75.0),
            ("anthropic", m) if m.contains("haiku") => (0.80, 4.0),

            // OpenAI
            ("openai", m) if m.contains("gpt-4") => (30.0, 60.0),
            ("openai", m) if m.contains("gpt-3.5") => (0.50, 1.50),
            ("openai", _) => (10.0, 30.0),

            // Groq (much cheaper)
            ("groq", m) if m.contains("70b") => (0.59, 0.59),
            ("groq", m) if m.contains("8x7b") => (0.27, 0.27),
            ("groq", m) if m.contains("9b") => (0.08, 0.08),
            ("groq", _) => (0.59, 0.59),

            // Default fallback
            _ => (1.0, 2.0),
        };

        // Cost per 1M tokens, convert to actual usage
        let prompt_cost = (self.prompt_tokens as f64 / 1_000_000.0) * input_cost;
        let completion_cost = (self.completion_tokens as f64 / 1_000_000.0) * output_cost;

        prompt_cost + completion_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            ChannelType::Telegram,
            "12345".to_string(),
            "user".to_string(),
            "Hello".to_string(),
        );

        assert_eq!(msg.channel_type, ChannelType::Telegram);
        assert_eq!(msg.channel_id, "12345");
        assert_eq!(msg.sender, "user");
        assert_eq!(msg.text, "Hello");
    }

    #[test]
    fn test_message_with_attachment() {
        let msg = Message::new(
            ChannelType::Discord,
            "67890".to_string(),
            "user2".to_string(),
            "World".to_string(),
        )
        .add_attachment(Attachment::new(
            "http://example.com/file.pdf".to_string(),
            "application/pdf".to_string(),
            1024,
        ));

        assert_eq!(msg.metadata.attachments.len(), 1);
        assert_eq!(msg.metadata.attachments[0].size_bytes, 1024);
    }

    #[test]
    fn test_message_with_reply_to() {
        let msg = Message::new(
            ChannelType::Telegram,
            "12345".to_string(),
            "user".to_string(),
            "Reply".to_string(),
        )
        .set_reply_to("message-123".to_string());

        assert_eq!(msg.metadata.reply_to, Some("message-123".to_string()));
    }

    #[test]
    fn test_channel_type_from_str() {
        assert_eq!(ChannelType::from_str("telegram"), Some(ChannelType::Telegram));
        assert_eq!(ChannelType::from_str("TELEGRAM"), Some(ChannelType::Telegram));
        assert_eq!(ChannelType::from_str("discord"), Some(ChannelType::Discord));
        assert_eq!(ChannelType::from_str("unknown"), None);
    }

    #[test]
    fn test_usage_stats_calculation() {
        let stats = UsageStats::new(1000, 500);
        assert_eq!(stats.prompt_tokens, 1000);
        assert_eq!(stats.completion_tokens, 500);
        assert_eq!(stats.total_tokens, 1500);
    }

    #[test]
    fn test_cost_calculation_groq() {
        let stats = UsageStats::new(1000, 500);
        let cost = stats.calculate_cost("groq", "llama-3.3-70b-versatile");

        // Groq: $0.59 per 1M tokens
        // (1000 + 500) / 1,000,000 * 0.59
        let expected = 1500.0 / 1_000_000.0 * 0.59;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_anthropic() {
        let stats = UsageStats::new(1000, 500);
        let cost = stats.calculate_cost("anthropic", "claude-sonnet-4");

        // Anthropic Sonnet: $3 input, $15 output per 1M tokens
        let expected = (1000.0 / 1_000_000.0 * 3.0) + (500.0 / 1_000_000.0 * 15.0);
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new(
            ChannelType::Discord,
            "test-channel".to_string(),
            "test-user".to_string(),
            "Test message".to_string(),
        );

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.text, deserialized.text);
        assert_eq!(msg.channel_type, deserialized.channel_type);
    }
}
