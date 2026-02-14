use crate::types::{Agent, AgentError, AgentMessage, AgentResponse, StreamChunk};
use async_trait::async_trait;

/// Placeholder agent for testing and development
pub struct PlaceholderAgent {
    config: clanker_config::AgentConfig,
}

impl PlaceholderAgent {
    pub fn new(config: clanker_config::AgentConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for PlaceholderAgent {
    async fn chat(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError> {
        let last_message = messages
            .last()
            .map(|msg| msg.content.clone())
            .unwrap_or_else(|| "Hello!".to_string());

        Ok(AgentResponse {
            content: format!(
                "Placeholder response from {}: {}",
                self.config.provider, last_message
            ),
            finish_reason: "stop".to_string(),
            usage: crate::types::Usage {
                prompt_tokens: messages.len() as u32,
                completion_tokens: 10,
                total_tokens: messages.len() as u32 + 10,
            },
            model: self.config.model.clone(),
            provider: self.config.provider.clone(),
        })
    }

    async fn chat_stream(
        &self,
        _messages: Vec<AgentMessage>,
    ) -> Result<
        Box<dyn futures::Stream<Item = Result<StreamChunk, AgentError>> + Send + Unpin>,
        AgentError,
    > {
        Err(AgentError::Unknown("Streaming not implemented for placeholder".to_string()))
    }

    fn provider(&self) -> &str {
        &self.config.provider
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clanker_config::AgentConfig;

    #[tokio::test]
    async fn test_placeholder_agent() {
        let config = clanker_config::AgentConfig {
            provider: "placeholder".to_string(),
            model: "test-model".to_string(),
            api_key_env: "PLACEHOLDER_API_KEY".to_string(),
            api_key: Some("test-key".to_string()),
            max_tokens: 100,
            api_base_url: None,
            worker: None,
        };

        let agent = PlaceholderAgent::new(config);

        let messages = vec![AgentMessage {
            role: crate::types::MessageRole::User,
            content: "Hello!".to_string(),
        }];

        let response = agent.chat(messages).await.unwrap();
        assert!(response.content.contains("Placeholder response"));
        assert_eq!(response.provider, "placeholder");
        assert_eq!(response.model, "test-model");
    }

    #[test]
    fn test_placeholder_creation() {
        let config = clanker_config::AgentConfig {
            provider: "placeholder".to_string(),
            model: "test-model".to_string(),
            api_key_env: "PLACEHOLDER_API_KEY".to_string(),
            api_key: Some("test-key".to_string()),
            max_tokens: 100,
            api_base_url: None,
            worker: None,
        };

        let agent = PlaceholderAgent::new(config);
        assert_eq!(agent.provider(), "placeholder");
        assert_eq!(agent.model(), "test-model");
    }
}
