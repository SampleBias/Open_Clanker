//! Clanker AI Agent Crate
//!
//! Provides AI agent implementations for multiple providers:
//! - Anthropic Claude
//! - OpenAI GPT
//! - Grok (xAI)
//! - Groq
//!
//! # Example
//!
//! ```no_run
//! use clanker_agent::{AgentFactory, Agent, AgentMessage, AgentConfig};
//! use clanker_agent::types::MessageRole;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AgentConfig {
//!         provider: "anthropic".to_string(),
//!         model: "claude-sonnet-4-20250514".to_string(),
//!         api_key_env: "OPENCLAW_ANTHROPIC_API_KEY".to_string(),
//!         api_key: Some("your-api-key".to_string()),
//!         max_tokens: 4096,
//!         api_base_url: None,
//!         worker: None,
//!         fallback: None,
//!     };
//!
//!     let agent = AgentFactory::create_from_config(config);
//!
//!     let messages = vec![
//!         AgentMessage {
//!             role: MessageRole::User,
//!             content: "Hello!".to_string(),
//!         }
//!     ];
//!
//!     let response = agent.chat(messages).await?;
//!     println!("Response: {}", response.content);
//!
//!     Ok(())
//! }
//! ```

pub mod anthropic;
pub mod factory;
pub mod grok;
pub mod groq;
pub mod openai;
pub mod orchestrator;
pub mod placeholder;
pub mod types;
pub mod zai;

// Re-exports for convenience
pub use factory::AgentFactory;
pub use orchestrator::{MasterClanker, MASTER_SYSTEM_PROMPT};
pub use types::{
    Agent, AgentError, AgentMessage, AgentResponse, MessageRole,
    StreamChunk, SystemPrompt, Usage, WorkerResult, WorkerTask, system_prompts,
};
pub use clanker_config::AgentConfig;
