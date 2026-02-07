use crate::anthropic::AnthropicAgent;
use crate::grok::GrokAgent;
use crate::groq::GroqAgent;
use crate::openai::OpenAIAgent;
use crate::types::Agent;
use clanker_config::AgentConfig;
use tracing::{debug, info};

/// Agent factory for creating provider-specific agents
pub struct AgentFactory;

impl AgentFactory {
    /// Create an agent from provider configuration
    pub fn create_from_config(config: AgentConfig) -> Box<dyn Agent> {
        info!(
            "Creating {} agent for model {}",
            config.provider,
            config.model
        );

        match config.provider.to_lowercase().as_str() {
            "anthropic" => {
                debug!("Creating Anthropic agent");
                Box::new(AnthropicAgent::new(config))
            }
            "openai" => {
                debug!("Creating OpenAI agent");
                Box::new(OpenAIAgent::new(config))
            }
            "grok" => {
                debug!("Creating Grok (xAI) agent");
                Box::new(GrokAgent::new(config))
            }
            "groq" => {
                debug!("Creating Groq agent");
                Box::new(GroqAgent::new(config))
            }
            _ => {
                debug!("Unknown provider, using placeholder agent");
                Box::new(crate::placeholder::PlaceholderAgent::new(config))
            }
        }
    }

    /// Get supported providers
    pub fn supported_providers() -> Vec<&'static str> {
        vec!["anthropic", "openai", "grok", "groq"]
    }

    /// Check if provider is supported
    pub fn is_supported(provider: &str) -> bool {
        Self::supported_providers()
            .contains(&provider.to_lowercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = AgentFactory::supported_providers();
        assert_eq!(providers.len(), 4);
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"grok"));
        assert!(providers.contains(&"groq"));
    }

    #[test]
    fn test_provider_support() {
        assert!(AgentFactory::is_supported("anthropic"));
        assert!(AgentFactory::is_supported("openai"));
        assert!(AgentFactory::is_supported("grok"));
        assert!(AgentFactory::is_supported("groq"));

        assert!(!AgentFactory::is_supported("unknown"));
        assert!(!AgentFactory::is_supported(""));
    }
}
