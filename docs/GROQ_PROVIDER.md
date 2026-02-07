# Groq Model Provider Implementation

## Overview

Groq provides ultra-fast inference with their LPU (Language Processing Unit) chips and uses the OpenAI-compatible API format, making it easy to integrate.

## Configuration

Add to `clanker-config`:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct AgentConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    pub max_tokens: u32,
    pub api_base_url: Option<String>,  // NEW: For custom endpoints
}
```

## Groq Client Implementation

Create `crates/agent/src/groq.rs`:

```rust
use crate::openai::*;  // Reuse OpenAI structures
use clanker_core::{traits::Agent, types::{AgentResponse, Message, UsageStats}, Result};
use reqwest::Client;
use std::time::Duration;

/// Groq API client (OpenAI-compatible)
///
/// Groq offers ultra-fast inference using LPU chips
/// https://console.groq.com/docs/api-reference
pub struct GroqAgent {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    api_base: String,
}

impl GroqAgent {
    /// Create a new Groq agent
    ///
    /// # Arguments
    /// * `api_key` - Groq API key (get from console.groq.com)
    /// * `model` - Model name (e.g., "llama-3.3-70b-versatile", "mixtral-8x7b-32768")
    /// * `max_tokens` - Maximum tokens in response
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self::with_base_url(api_key, model, max_tokens, "https://api.groq.com/openai/v1".to_string())
    }

    /// Create Groq agent with custom base URL (for testing/custom endpoints)
    pub fn with_base_url(api_key: String, model: String, max_tokens: u32, api_base: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            client,
            api_key,
            model,
            max_tokens,
            api_base,
        }
    }

    /// Get available Groq models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .get(&format!("{}/models", self.api_base))
            .bearer_auth(&self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(clanker_core::ClankerError::Agent {
                provider: "groq".to_string(),
                message: format!("Failed to list models: {}", error_text),
            });
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }

        #[derive(Deserialize)]
        struct ModelData {
            id: String,
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.data.iter().map(|m| m.id.clone()).collect())
    }
}

#[async_trait::async_trait]
impl Agent for GroqAgent {
    async fn generate(&self, messages: &[Message]) -> Result<AgentResponse> {
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: "user".to_string(),
                content: msg.text.clone(),
            })
            .collect();

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: Some(0.7),
            max_tokens: Some(self.max_tokens),
            top_p: Some(0.9),
            stream: Some(false),
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.api_base))
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(clanker_core::ClankerError::Agent {
                provider: "groq".to_string(),
                message: error_text,
            });
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let content = openai_response
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AgentResponse {
            id: openai_response.id,
            content,
            model: openai_response.model,
            finish_reason: openai_response.choices.get(0)
                .map(|c| c.finish_reason.clone())
                .unwrap_or("stop".to_string()),
            usage: UsageStats {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
        })
    }

    async fn health(&self) -> Result<bool> {
        // Simple health check by trying to list models
        match self.list_models().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

// Reuse OpenAI request/response structures
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageContent,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

/// Available Groq models
///
/// See: https://console.groq.com/docs/models
impl GroqAgent {
    /// Llama 3.3 70B Versatile
    /// Great for general-purpose tasks, coding, and reasoning
    pub const MODEL_LLAMA_3_3_70B: &'static str = "llama-3.3-70b-versatile";

    /// Llama 3.1 70B Versatile
    /// Fast and efficient for most tasks
    pub const MODEL_LLAMA_3_1_70B: &'static str = "llama-3.1-70b-versatile";

    /// Mixtral 8x7B-32768
    /// High-quality reasoning with 32k context
    pub const MODEL_MIXTRAL_8X7B: &'static str = "mixtral-8x7b-32768";

    /// Gemma2 9B It
    /// Google's Gemma model optimized for Groq
    pub const MODEL_GEMMA2_9B: &'static str = "gemma2-9b-it";

    /// Get recommended model for use case
    ///
    /// # Arguments
    /// * `task_type` - Type of task: "general", "coding", "fast", "reasoning"
    pub fn recommended_model(task_type: &str) -> &'static str {
        match task_type {
            "general" | "" => Self::MODEL_LLAMA_3_3_70B,
            "coding" => Self::MODEL_LLAMA_3_3_70B,
            "fast" => Self::MODEL_GEMMA2_9B,
            "reasoning" => Self::MODEL_LLAMA_3_3_70B,
            _ => Self::MODEL_LLAMA_3_3_70B,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_models() {
        assert_eq!(GroqAgent::recommended_model("general"), GroqAgent::MODEL_LLAMA_3_3_70B);
        assert_eq!(GroqAgent::recommended_model("coding"), GroqAgent::MODEL_LLAMA_3_3_70B);
        assert_eq!(GroqAgent::recommended_model("fast"), GroqAgent::MODEL_GEMMA2_9B);
        assert_eq!(GroqAgent::recommended_model("reasoning"), GroqAgent::MODEL_LLAMA_3_3_70B);
    }
}
```

## Agent Factory

Update `crates/agent/src/lib.rs` to include Groq:

```rust
use clanker_core::{traits::Agent, types::Message, Result};
use anyhow::Context;
use std::sync::Arc;

pub mod anthropic;
pub mod openai;
pub mod groq;  // NEW: Groq module

/// Factory for creating agent instances
pub struct AgentFactory;

impl AgentFactory {
    /// Create agent from configuration
    ///
    /// # Supported Providers
    /// * `anthropic` - Anthropic Claude models
    /// * `openai` - OpenAI GPT models
    /// * `groq` - Groq ultra-fast inference
    pub fn create(
        provider: &str,
        api_key: String,
        model: String,
        max_tokens: u32,
        api_base: Option<String>,
    ) -> Result<Arc<dyn Agent>> {
        match provider.to_lowercase().as_str() {
            "anthropic" => Ok(Arc::new(anthropic::AnthropicAgent::new(
                api_key,
                model,
                max_tokens,
            ))),
            "openai" => Ok(Arc::new(openai::OpenAIAgent::new(api_key, model))),
            "groq" => {
                let base_url = api_base.unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string());
                Ok(Arc::new(groq::GroqAgent::with_base_url(
                    api_key,
                    model,
                    max_tokens,
                    base_url,
                )))
            }
            _ => Err(clanker_core::ClankerError::Config(format!(
                "Unsupported provider: {}. Supported: anthropic, openai, groq",
                provider
            ))),
        }
    }
}
```

## Configuration Examples

### config.toml with Groq

```toml
[agent]
provider = "groq"
model = "llama-3.3-70b-versatile"
api_key_env = "OPENCLAW_GROQ_API_KEY"
max_tokens = 4096

[agent]
# Alternative: For ultra-fast responses
provider = "groq"
model = "gemma2-9b-it"
api_key_env = "OPENCLAW_GROQ_API_KEY"
max_tokens = 2048
```

### .env

```bash
# Groq API Key (get from console.groq.com)
OPENCLAW_GROQ_API_KEY=gsk_your_key_here

# Optional: Custom API endpoint
OPENCLAW_GROQ_API_BASE=https://api.groq.com/openai/v1
```

## Groq Model Comparison

| Model | Context | Speed | Best For |
|-------|---------|-------|----------|
| `llama-3.3-70b-versatile` | 128k | Very Fast | General, coding, reasoning |
| `llama-3.1-70b-versatile` | 128k | Ultra Fast | General purpose |
| `mixtral-8x7b-32768` | 32k | Very Fast | Reasoning, complex tasks |
| `gemma2-9b-it` | 8k | Extremely Fast | Quick responses, simple tasks |

## Performance Expectations

Groq's LPU architecture provides significant speed advantages:

| Metric | OpenAI GPT-4 | Groq Llama 3.3 | Improvement |
|--------|---------------|-----------------|-------------|
| Latency (50 tokens) | ~2-3s | ~100-200ms | **15-30x faster** |
| Throughput | ~50 tokens/s | ~500+ tokens/s | **10x faster** |
| Cost per 1M tokens | $30 | $0.59 | **50x cheaper** |

## CLI Usage

```bash
# Generate config with Groq
open-clanker config-generate

# Edit config.toml
[agent]
provider = "groq"
model = "llama-3.3-70b-versatile"
api_key_env = "OPENCLAW_GROQ_API_KEY"
max_tokens = 4096

# Run with Groq
OPENCLAW_GROQ_API_KEY=gsk_your_key open-clanker gateway
```

## Testing

Add tests for Groq integration:

```rust
// crates/agent/src/groq/tests.rs

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore]  // Run with: cargo test -- --ignored
    async fn test_groq_generate() {
        let api_key = std::env::var("GROQ_TEST_API_KEY")
            .expect("GROQ_TEST_API_KEY must be set for integration tests");

        let agent = GroqAgent::new(api_key, "llama-3.3-70b-versatile".to_string(), 100);

        let messages = vec![Message::new(
            clanker_core::types::ChannelType::Telegram,
            "test".to_string(),
            "user".to_string(),
            "Say 'Hello from Groq!' in exactly those words.".to_string(),
        )];

        let response = agent.generate(&messages).await.unwrap();

        assert!(response.content.contains("Hello from Groq!"));
        assert!(response.usage.total_tokens > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_groq_list_models() {
        let api_key = std::env::var("GROQ_TEST_API_KEY")
            .expect("GROQ_TEST_API_KEY must be set for integration tests");

        let agent = GroqAgent::new(api_key, "llama-3.3-70b-versatile".to_string(), 100);

        let models = agent.list_models().await.unwrap();

        assert!(models.len() > 0);
        assert!(models.iter().any(|m| m.contains("llama")));
    }
}
```

## Getting Groq API Key

1. Go to [console.groq.com](https://console.groq.com)
2. Create an account (free tier available)
3. Navigate to API Keys
4. Create a new API key
5. Copy the key (starts with `gsk_`)
6. Add to `.env`:

```bash
OPENCLAW_GROQ_API_KEY=gsk_your_actual_key_here
```

## Rate Limits

Groq's free tier includes:
- 30 requests per minute
- 14,400 tokens per minute
- 500,000 tokens per day

Paid tiers offer higher limits. See [Groq Pricing](https://console.groq.com/settings/billing).

## Error Handling

Groq returns standard HTTP status codes:

| Status | Meaning | Action |
|--------|---------|--------|
| 401 | Invalid API key | Check API key |
| 429 | Rate limit exceeded | Implement backoff |
| 500 | Internal error | Retry with exponential backoff |
| 503 | Service unavailable | Wait and retry |

Implement exponential backoff in production:

```rust
// Example: Retry logic
use tokio::time::{sleep, Duration};

async fn generate_with_retry(
    agent: &GroqAgent,
    messages: &[Message],
    max_retries: u32,
) -> Result<AgentResponse> {
    let mut delay = Duration::from_millis(100);

    for attempt in 0..max_retries {
        match agent.generate(messages).await {
            Ok(response) => return Ok(response),
            Err(err) if attempt < max_retries - 1 => {
                tracing::warn!("Attempt {} failed, retrying in {:?}", attempt + 1, delay);
                sleep(delay).await;
                delay *= 2;  // Exponential backoff
            }
            Err(err) => return Err(err),
        }
    }

    unreachable!()
}
```

## Best Practices

1. **Model Selection**
   - Use `llama-3.3-70b-versatile` for general tasks
   - Use `gemma2-9b-it` for ultra-fast responses
   - Consider context window needs (8k vs 32k vs 128k)

2. **Token Management**
   - Set appropriate `max_tokens` based on task
   - Monitor usage with `response.usage`
   - Consider batch processing for multiple requests

3. **Error Handling**
   - Implement retry logic with exponential backoff
   - Handle rate limits gracefully
   - Log errors for debugging

4. **Performance**
   - Use streaming for long responses (future enhancement)
   - Batch similar requests when possible
   - Cache responses when appropriate

## Future Enhancements

- [ ] Streaming support for real-time responses
- [ ] Function calling support when available
- [ ] Vision/multimodal models when supported
- [ ] Custom model fine-tuning
- [ ] Batch API support

## Resources

- [Groq Documentation](https://console.groq.com/docs)
- [Groq Models](https://console.groq.com/docs/models)
- [Groq API Reference](https://console.groq.com/docs/api-reference)
- [Groq Pricing](https://console.groq.com/settings/billing)
- [Groq Discord Community](https://discord.gg/groq)

## Summary

Groq provides an excellent addition to Open Clanker's AI providers with:
- **Ultra-fast inference** (15-30x faster than OpenAI)
- **Cost-effective** (50x cheaper than GPT-4)
- **OpenAI-compatible API** (easy integration)
- **Multiple model options** (Llama, Mixtral, Gemma)
- **Free tier available** (for testing)

The integration leverages the existing OpenAI client structure, making it simple to add and maintain.
