use crate::types::{AgentResponse, ChannelType, Message};
use crate::Result;
use async_trait::async_trait;

/// Trait for message channels (Telegram, Discord, etc.)
#[async_trait]
pub trait Channel: Send + Sync {
    /// Get channel type identifier
    fn channel_type(&self) -> ChannelType;

    /// Get channel name (for logging)
    fn channel_name(&self) -> &str {
        self.channel_type().as_str()
    }

    /// Send a message through this channel
    async fn send_message(&self, message: &Message) -> Result<()>;

    /// Start listening for incoming messages
    async fn listen(&self) -> Result<()>;

    /// Stop listening for incoming messages
    async fn stop(&self) -> Result<()>;

    /// Check if channel is healthy/connected
    async fn health(&self) -> Result<bool> {
        Ok(true)
    }

    /// Get unique channel identifier (e.g., bot ID)
    fn channel_id(&self) -> String;
}

/// Trait for AI agents (Anthropic, OpenAI, Groq, etc.)
#[async_trait]
pub trait Agent: Send + Sync {
    /// Generate a response from AI agent
    ///
    /// # Arguments
    /// * `messages` - Slice of messages to process
    ///
    /// # Returns
    /// Agent response with content and usage statistics
    async fn generate(&self, messages: &[Message]) -> Result<AgentResponse>;

    /// Check if agent is healthy and can make API calls
    async fn health(&self) -> Result<bool>;

    /// Get agent provider name (e.g., "anthropic", "openai", "groq")
    fn provider_name(&self) -> &str;

    /// Get model name being used
    fn model_name(&self) -> &str;

    /// Get maximum context window (tokens)
    fn max_context(&self) -> u32 {
        4096 // Default
    }

    /// Check if agent supports streaming responses
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// Trait for persistent storage
#[async_trait]
pub trait Storage: Send + Sync {
    /// Save a message to storage
    async fn save_message(&self, message: &Message) -> Result<()>;

    /// Get messages for a specific channel
    async fn get_messages(&self, channel_id: &str, limit: usize) -> Result<Vec<Message>>;

    /// Get a specific message by ID
    async fn get_message(&self, message_id: &str) -> Result<Option<Message>>;

    /// Delete messages older than specified duration
    async fn prune_messages(&self, older_than: chrono::Duration) -> Result<u64>;

    /// Initialize storage (create tables, etc.)
    async fn initialize(&self) -> Result<()>;

    /// Check storage health
    async fn health(&self) -> Result<bool> {
        Ok(true)
    }
}

/// Trait for configuration management
pub trait Configurable: Send + Sync {
    /// Load configuration from file
    fn load_from_path<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()>;

    /// Load configuration from environment variables
    fn load_from_env(&mut self) -> Result<()>;

    /// Validate configuration
    fn validate(&self) -> Result<()>;

    /// Save configuration to file
    fn save_to_path<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()>;
}

/// Trait for metrics and monitoring
pub trait Metrics: Send + Sync {
    /// Record a message sent
    fn record_message_sent(&self, channel: ChannelType);

    /// Record a message received
    fn record_message_received(&self, channel: ChannelType);

    /// Record an AI generation
    fn record_ai_generation(&self, provider: &str, model: &str, tokens: u32, latency_ms: u64);

    /// Record an error
    fn record_error(&self, error_type: &str, source: &str);

    /// Increment counter
    fn increment_counter(&self, name: &str, value: u64);

    /// Record timing
    fn record_timing(&self, name: &str, duration_ms: u64);

    /// Record gauge value
    fn record_gauge(&self, name: &str, value: f64);
}

/// Trait for health checking
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Check if component is healthy
    async fn health(&self) -> Result<bool>;

    /// Get health status description
    fn status(&self) -> HealthStatus;

    /// Get component name
    fn name(&self) -> &str;
}

/// Health status for components
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but functioning
    Degraded(String),
    /// Component is unhealthy
    Unhealthy(String),
}

impl HealthStatus {
    /// Check if status is healthy (not degraded or unhealthy)
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if status is functional (healthy or degraded)
    pub fn is_functional(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded(_))
    }

    /// Create healthy status
    pub fn healthy() -> Self {
        Self::Healthy
    }

    /// Create degraded status
    pub fn degraded(reason: impl Into<String>) -> Self {
        Self::Degraded(reason.into())
    }

    /// Create unhealthy status
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        Self::Unhealthy(reason.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(HealthStatus::Healthy.is_functional());
        assert!(!HealthStatus::Degraded("slow".to_string()).is_healthy());
        assert!(HealthStatus::Degraded("slow".to_string()).is_functional());
        assert!(!HealthStatus::Unhealthy("down".to_string()).is_healthy());
        assert!(!HealthStatus::Unhealthy("down".to_string()).is_functional());
    }

    #[test]
    fn test_health_status_creation() {
        let healthy = HealthStatus::healthy();
        assert_eq!(healthy, HealthStatus::Healthy);

        let degraded = HealthStatus::degraded("slow response");
        assert_eq!(degraded, HealthStatus::Degraded("slow response".to_string()));

        let unhealthy = HealthStatus::unhealthy("connection failed");
        assert_eq!(unhealthy, HealthStatus::Unhealthy("connection failed".to_string()));
    }
}
