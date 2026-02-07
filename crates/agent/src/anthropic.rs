use crate::types::{
    Agent, AgentError, AgentMessage, AgentResponse, StreamChunk, Usage,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, trace};

/// Anthropic Claude agent
pub struct AnthropicAgent {
    config: clanker_config::AgentConfig,
    client: Client,
}

impl AnthropicAgent {
    pub fn new(config: clanker_config::AgentConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    const API_URL: &'static str = "https://api.anthropic.com/v1/messages";
}

#[async_trait]
impl Agent for AnthropicAgent {
    async fn chat(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError> {
        debug!("Sending chat request to Anthropic");

        let request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            system: "You are a helpful AI assistant.".to_string(),
            messages: messages_to_anthropic(messages),
        };

        let response = self
            .client
            .post(Self::API_URL)
            .header("x-api-key", self.config.api_key.as_ref().unwrap_or(&String::new()))
            .header("anthropic-version", "2023-06-01")
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
                "Anthropic API error {}: {}",
                status, response_text
            )));
        }

        let anthropic_response: AnthropicResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;

        trace!("Received Anthropic response");

        Ok(AgentResponse {
            content: anthropic_response.content[0].text.clone(),
            finish_reason: anthropic_response.stop_reason.unwrap_or_else(|| "stop".to_string()),
            usage: Usage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            },
            model: self.config.model.clone(),
            provider: "anthropic".to_string(),
        })
    }

    async fn chat_stream(
        &self,
        _messages: Vec<AgentMessage>,
    ) -> Result<
        Box<dyn futures::Stream<Item = Result<StreamChunk, AgentError>> + Send + Unpin>,
        AgentError,
    > {
        debug!("Streaming not yet implemented for Anthropic");
        Err(AgentError::Unknown("Streaming not implemented".to_string()))
    }

    fn provider(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Anthropic API request
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Convert agent messages to Anthropic format
fn messages_to_anthropic(messages: Vec<AgentMessage>) -> Vec<AnthropicMessage> {
    messages
        .into_iter()
        .map(|msg| AnthropicMessage {
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
    fn test_messages_to_anthropic() {
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

        let anthropic_messages = messages_to_anthropic(messages);

        assert_eq!(anthropic_messages.len(), 2);
        assert_eq!(anthropic_messages[0].role, "user");
        assert_eq!(anthropic_messages[0].content, "Hello");
        assert_eq!(anthropic_messages[1].role, "assistant");
        assert_eq!(anthropic_messages[1].content, "Hi there!");
    }
}
