use crate::types::{
    Agent, AgentError, AgentMessage, AgentResponse, StreamChunk, Usage,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, trace};

/// Grok (xAI) agent
pub struct GrokAgent {
    config: clanker_config::AgentConfig,
    client: Client,
}

impl GrokAgent {
    pub fn new(config: clanker_config::AgentConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    const API_URL: &'static str = "https://api.x.ai/v1/chat/completions";
}

#[async_trait]
impl Agent for GrokAgent {
    async fn chat(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError> {
        debug!("Sending chat request to Grok (xAI)");

        let request = GrokRequest {
            model: self.config.model.clone(),
            messages: messages_to_grok(messages),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(0.7), // Default temperature
        };

        let response = self
            .client
            .post(Self::API_URL)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_ref().unwrap_or(&String::new())),
            )
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::RequestFailed(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AgentError::HttpError(e.to_string()))?;

        if !status.is_success() {
            return Err(AgentError::ProviderError(format!(
                "Grok API error {}: {}",
                status, response_text
            )));
        }

        let grok_response: GrokResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;

        trace!("Received Grok response");

        let content = grok_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "".to_string());

        let finish_reason = grok_response
            .choices
            .first()
            .map(|c| c.finish_reason.clone())
            .unwrap_or_else(|| "stop".to_string());

        Ok(AgentResponse {
            content,
            finish_reason,
            usage: Usage {
                prompt_tokens: grok_response.usage.prompt_tokens,
                completion_tokens: grok_response.usage.completion_tokens,
                total_tokens: grok_response.usage.total_tokens,
            },
            model: self.config.model.clone(),
            provider: "grok".to_string(),
        })
    }

    async fn chat_stream(
        &self,
        _messages: Vec<AgentMessage>,
    ) -> Result<
        Box<dyn futures::Stream<Item = Result<StreamChunk, AgentError>> + Send + Unpin>,
        AgentError,
    > {
        debug!("Streaming not yet implemented for Grok");
        Err(AgentError::Unknown("Streaming not implemented".to_string()))
    }

    fn provider(&self) -> &str {
        "grok"
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Grok API request (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct GrokRequest {
    model: String,
    messages: Vec<GrokMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct GrokMessage {
    role: String,
    content: String,
}

/// Grok API response (OpenAI-compatible)
#[derive(Debug, Deserialize)]
struct GrokResponse {
    choices: Vec<Choice>,
    usage: GrokUsage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GrokUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Convert agent messages to Grok format
fn messages_to_grok(messages: Vec<AgentMessage>) -> Vec<GrokMessage> {
    messages
        .into_iter()
        .map(|msg| GrokMessage {
            role: serde_json::to_string(&msg.role)
                .unwrap_or_else(|_| "\"user\"".to_string())
                .trim_matches('"')
                .to_string(),
            content: msg.content,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_to_grok() {
        use crate::types::MessageRole;
        let messages = vec![
            AgentMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
            },
        ];

        let grok_messages = messages_to_grok(messages);

        assert_eq!(grok_messages.len(), 2);
        assert_eq!(grok_messages[0].role, "user");
        assert_eq!(grok_messages[0].content, "Hello");
        assert_eq!(grok_messages[1].role, "assistant");
        assert_eq!(grok_messages[1].content, "Hi there!");
    }

    #[test]
    fn test_grok_agent_creation() {
        let config = clanker_config::AgentConfig {
            provider: "grok".to_string(),
            model: "grok-2".to_string(),
            api_key_env: "GROK_API_KEY".to_string(),
            api_key: Some("test-key".to_string()),
            max_tokens: 100,
            api_base_url: None,
            worker: None,
        };

        let agent = GrokAgent::new(config);
        assert_eq!(agent.provider(), "grok");
        assert_eq!(agent.model(), "grok-2");
    }
}
