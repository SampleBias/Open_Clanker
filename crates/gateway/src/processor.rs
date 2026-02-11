//! Message processing: core::Message → Agent → core::Message

use clanker_agent::{Agent, AgentFactory, AgentMessage, MessageRole};
use clanker_core::Message;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Process incoming message through agent and return AI response
pub async fn process_message(
    agent: &(dyn Agent + Send + Sync),
    incoming: &Message,
) -> Result<Message, String> {
    let user_content = incoming.text.clone();
    if user_content.is_empty() {
        return Err("Message text cannot be empty".to_string());
    }

    info!(
        "Processing message from {} ({}): {} chars",
        incoming.sender,
        incoming.channel_type,
        user_content.len()
    );

    let agent_messages = vec![AgentMessage {
        role: MessageRole::User,
        content: user_content,
    }];

    let response = agent
        .chat(agent_messages)
        .await
        .map_err(|e| {
            error!("Agent error: {}", e);
            e.to_string()
        })?;

    debug!(
        "Agent response: {} chars, model={}",
        response.content.len(),
        response.model
    );

    let response_message = Message::new(
        incoming.channel_type,
        incoming.channel_id.clone(),
        "assistant".to_string(),
        response.content,
    );

    Ok(response_message)
}

/// Create agent from config
pub fn create_agent(config: &clanker_config::Config) -> Arc<dyn Agent + Send + Sync> {
    AgentFactory::create_arc_from_config(config.agent.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clanker_agent::AgentFactory;
    use clanker_config::AgentConfig;
    use clanker_core::ChannelType;

    fn create_placeholder_agent() -> Arc<dyn Agent + Send + Sync> {
        AgentFactory::create_arc_from_config(AgentConfig {
            provider: "placeholder".to_string(),
            model: "test".to_string(),
            api_key_env: "TEST".to_string(),
            api_key: None,
            max_tokens: 100,
            api_base_url: None,
        })
    }

    #[tokio::test]
    async fn test_process_message_empty_fails() {
        let agent = create_placeholder_agent();
        let msg = Message::new(
            ChannelType::Telegram,
            "123".to_string(),
            "user".to_string(),
            "".to_string(),
        );

        let result = process_message(agent.as_ref(), &msg).await;
        assert!(result.is_err());
    }
}
