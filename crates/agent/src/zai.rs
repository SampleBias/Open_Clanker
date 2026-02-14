//! Z.ai GLM-4.7 agent (OpenAI-compatible API)
//!
//! Z.ai provides GLM-4.7 with strong coding, tool invocation, and agent capabilities.
//! API: https://api.z.ai/api/paas/v4/chat/completions
//! Docs: https://docs.z.ai/

use crate::types::{
    Agent, AgentError, AgentMessage, AgentResponse, StreamChunk, Usage,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, trace};

/// Z.ai GLM-4.7 agent (OpenAI-compatible)
pub struct ZaiAgent {
    config: clanker_config::AgentConfig,
    client: Client,
}

impl ZaiAgent {
    pub fn new(config: clanker_config::AgentConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Z.ai API base URL (OpenAI-compatible)
    const API_BASE: &'static str = "https://api.z.ai/api/paas/v4";
}

#[async_trait]
impl Agent for ZaiAgent {
    async fn chat(&self, messages: Vec<AgentMessage>) -> Result<AgentResponse, AgentError> {
        debug!("Sending chat request to Z.ai (GLM-4.7)");

        let api_url = self
            .config
            .api_base_url
            .as_deref()
            .unwrap_or(Self::API_BASE);
        let url = format!("{}/chat/completions", api_url.trim_end_matches('/'));

        let request = ZaiRequest {
            model: self.config.model.clone(),
            messages: messages_to_zai(messages),
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(0.7),
        };

        let response = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_ref().unwrap_or(&String::new())),
            )
            .header("Content-Type", "application/json")
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
                "Z.ai API error {}: {}",
                status, response_text
            )));
        }

        let zai_response: ZaiResponse = serde_json::from_str(&response_text)
            .map_err(|e| AgentError::InvalidResponse(e.to_string()))?;

        trace!("Received Z.ai response");

        let content = zai_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_else(|| "".to_string());

        let finish_reason = zai_response
            .choices
            .first()
            .map(|c| c.finish_reason.clone().unwrap_or_else(|| "stop".to_string()))
            .unwrap_or_else(|| "stop".to_string());

        let usage = zai_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }).unwrap_or(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        Ok(AgentResponse {
            content,
            finish_reason,
            usage,
            model: self.config.model.clone(),
            provider: "zai".to_string(),
        })
    }

    async fn chat_stream(
        &self,
        _messages: Vec<AgentMessage>,
    ) -> Result<
        Box<dyn futures::Stream<Item = Result<StreamChunk, AgentError>> + Send + Unpin>,
        AgentError,
    > {
        debug!("Streaming not yet implemented for Z.ai");
        Err(AgentError::Unknown("Streaming not implemented".to_string()))
    }

    fn provider(&self) -> &str {
        "zai"
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Z.ai API request (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct ZaiRequest {
    model: String,
    messages: Vec<ZaiMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct ZaiMessage {
    role: String,
    content: String,
}

/// Z.ai API response (OpenAI-compatible)
#[derive(Debug, Deserialize)]
struct ZaiResponse {
    choices: Vec<ZaiChoice>,
    usage: Option<ZaiUsage>,
}

#[derive(Debug, Deserialize)]
struct ZaiChoice {
    message: ZaiMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZaiMessageResponse {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZaiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

fn messages_to_zai(messages: Vec<AgentMessage>) -> Vec<ZaiMessage> {
    messages
        .into_iter()
        .map(|msg| ZaiMessage {
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
    fn test_messages_to_zai() {
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

        let zai_messages = messages_to_zai(messages);

        assert_eq!(zai_messages.len(), 2);
        assert_eq!(zai_messages[0].role, "user");
        assert_eq!(zai_messages[0].content, "Hello");
        assert_eq!(zai_messages[1].role, "assistant");
        assert_eq!(zai_messages[1].content, "Hi there!");
    }

    #[test]
    fn test_zai_agent_creation() {
        let config = clanker_config::AgentConfig {
            provider: "zai".to_string(),
            model: "glm-4.7".to_string(),
            api_key_env: "ZAI_API_KEY".to_string(),
            api_key: Some("test-key".to_string()),
            max_tokens: 4096,
            api_base_url: None,
            worker: None,
            fallback: None,
        };

        let agent = ZaiAgent::new(config);
        assert_eq!(agent.provider(), "zai");
        assert_eq!(agent.model(), "glm-4.7");
    }
}
