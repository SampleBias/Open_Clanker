//! Open Clanker Core Library
//!
//! This library provides shared types, error handling, and traits
//! for the Open Clanker AI assistant gateway.

pub mod error;
pub mod traits;
pub mod types;

// Re-export common types
pub use error::{ClankerError, Result};
pub use traits::{
    Agent, Channel, Configurable, HealthCheck, HealthStatus, Metrics, Storage,
};
pub use types::{
    AgentResponse, Attachment, ChannelType, Message, MessageId, MessageMetadata,
    UsageStats, UserId,
};

// Re-export chrono and serde for convenience
pub use chrono;
pub use serde;
pub use serde_json;
pub use uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_imports() {
        // Verify all main types can be imported
        let _ = Message::new(
            ChannelType::Telegram,
            "test".to_string(),
            "user".to_string(),
            "test".to_string(),
        );

        let err = ClankerError::config("test");
        assert!(matches!(err, ClankerError::Config(_)));

        let status = HealthStatus::healthy();
        assert!(status.is_healthy());
    }
}
