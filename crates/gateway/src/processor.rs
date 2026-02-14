//! Message processing: core::Message → Agent → core::Message
//!
//! When orchestration is enabled, uses Master_Clanker which may delegate to Worker_Clankers.

use crate::state::AppState;
use clanker_agent::{Agent, AgentFactory, AgentMessage, MessageRole, MASTER_SYSTEM_PROMPT};
use clanker_core::Message;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Process incoming message through agent (or orchestrator) and return AI response
pub async fn process_message(state: &AppState, incoming: &Message) -> Result<Message, String> {
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

    let fallback = state.fallback_agent();
    let content = if state.orchestration_enabled() {
        if let Some(orchestrator) = state.orchestrator() {
            process_with_orchestration(state, orchestrator, fallback.as_deref(), &user_content).await?
        } else {
            process_direct(state.agent().as_ref(), fallback.as_deref(), &user_content).await?
        }
    } else {
        process_direct(state.agent().as_ref(), fallback.as_deref(), &user_content).await?
    };

    let response_message = Message::new(
        incoming.channel_type,
        incoming.channel_id.clone(),
        "assistant".to_string(),
        content,
    );

    Ok(response_message)
}

/// Direct agent call (no orchestration). Retries with fallback agent on failure.
async fn process_direct(
    agent: &(dyn Agent + Send + Sync),
    fallback: Option<&(dyn Agent + Send + Sync)>,
    user_content: &str,
) -> Result<String, String> {
    let agent_messages = vec![AgentMessage {
        role: MessageRole::User,
        content: user_content.to_string(),
    }];

    let result = agent.chat(agent_messages.clone()).await;
    if let Ok(response) = result {
        debug!(
            "Agent response: {} chars, model={}",
            response.content.len(),
            response.model
        );
        return Ok(response.content);
    }

    if let Some(fb) = fallback {
        error!("Primary agent failed, retrying with fallback ({})", fb.provider());
        let response = fb.chat(agent_messages).await.map_err(|e| {
            error!("Fallback agent error: {}", e);
            e.to_string()
        })?;
        debug!(
            "Fallback response: {} chars, model={}",
            response.content.len(),
            response.model
        );
        return Ok(response.content);
    }

    Err(result.unwrap_err().to_string())
}

/// Orchestration flow: Master_Clanker may delegate to Worker_Clankers.
/// Retries with fallback agent when master fails.
async fn process_with_orchestration(
    state: &AppState,
    orchestrator: &clanker_agent::MasterClanker,
    fallback: Option<&(dyn Agent + Send + Sync)>,
    user_content: &str,
) -> Result<String, String> {
    let master = orchestrator.master_agent();

    // First call: Master decides whether to delegate or respond directly
    let mut messages = vec![
        AgentMessage {
            role: MessageRole::System,
            content: MASTER_SYSTEM_PROMPT.to_string(),
        },
        AgentMessage {
            role: MessageRole::User,
            content: user_content.to_string(),
        },
    ];

    let response = match master.chat(messages.clone()).await {
        Ok(r) => r,
        Err(e) => {
            error!("Master_Clanker error: {}", e);
            if let Some(fb) = fallback {
                error!("Retrying with fallback ({})", fb.provider());
                return fb
                    .chat(messages)
                    .await
                    .map(|r| r.content)
                    .map_err(|e2| {
                        error!("Fallback agent error: {}", e2);
                        e2.to_string()
                    });
            }
            return Err(e.to_string());
        }
    };

    let master_response = response.content.trim();

    // Check for delegation
    if let Some(worker_tasks) = clanker_agent::MasterClanker::parse_delegation(master_response) {
        let n = worker_tasks.len().min(state.worker_max());

        if n == 0 {
            return Ok(master_response.to_string());
        }

        // Acquire semaphore permits before spawning
        let semaphore = state.worker_semaphore();
        let _permit = semaphore
            .acquire_many_owned(n as u32)
            .await
            .map_err(|e| {
                error!("Semaphore closed: {}", e);
                "Worker limit unavailable".to_string()
            })?;

        state.increment_worker_count(n);

        let worker_tasks: Vec<_> = worker_tasks.into_iter().take(n).collect();
        let results = orchestrator.delegate(worker_tasks).await;

        state.decrement_worker_count(n);

        // Second call: Master synthesizes worker results
        let results_text = results
            .iter()
            .map(|r| format!("[{}] Task: {}\nResult: {}", r.identity, r.task, r.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        messages.push(AgentMessage {
            role: MessageRole::Assistant,
            content: master_response.to_string(),
        });
        messages.push(AgentMessage {
            role: MessageRole::User,
            content: format!(
                "Worker_Clanker results:\n\n{}\n\nSynthesize these results into a coherent response for the user.",
                results_text
            ),
        });

        let synthesis = match master.chat(messages.clone()).await {
            Ok(r) => r,
            Err(e) => {
                error!("Master_Clanker synthesis error: {}", e);
                if let Some(fb) = fallback {
                    error!("Retrying synthesis with fallback ({})", fb.provider());
                    return fb
                        .chat(messages)
                        .await
                        .map(|r| r.content)
                        .map_err(|e2| {
                            error!("Fallback agent error: {}", e2);
                            e2.to_string()
                        });
                }
                return Err(e.to_string());
            }
        };

        Ok(synthesis.content)
    } else {
        // No delegation - Master's response is final
        Ok(master_response.to_string())
    }
}

/// Create agent from config
pub fn create_agent(config: &clanker_config::Config) -> Arc<dyn Agent + Send + Sync> {
    let mut agent_config = config.agent.clone();
    agent_config.worker = None;
    agent_config.fallback = None;
    AgentFactory::create_arc_from_config(agent_config)
}

/// Create fallback agent from config when configured and API key is present
pub fn create_fallback_agent(config: &clanker_config::Config) -> Option<Arc<dyn Agent + Send + Sync>> {
    let fallback = config.agent.fallback.as_ref()?;
    let api_key = fallback.api_key.as_ref().filter(|k| !k.is_empty())?;
    let agent_config = clanker_config::AgentConfig {
        provider: fallback.provider.clone(),
        model: fallback.model.clone(),
        api_key_env: fallback.api_key_env.clone(),
        api_key: Some(api_key.clone()),
        max_tokens: 4096,
        api_base_url: None,
        worker: None,
        fallback: None,
    };
    Some(AgentFactory::create_arc_from_config(agent_config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clanker_agent::AgentFactory;
    use clanker_config::Config;
    use clanker_core::ChannelType;
    use tokio_util::sync::CancellationToken;

    fn create_test_config_no_orchestration() -> Config {
        let mut config: Config = toml::from_str(include_str!("../../../config-examples/config.toml")).unwrap();
        config.orchestration.enabled = false;
        config
    }

    #[tokio::test]
    async fn test_process_message_empty_fails() {
        let config = create_test_config_no_orchestration();
        let shutdown = CancellationToken::new();
        let state = AppState::new(config, shutdown);

        let msg = Message::new(
            ChannelType::Telegram,
            "123".to_string(),
            "user".to_string(),
            "".to_string(),
        );

        let result = process_message(&state, &msg).await;
        assert!(result.is_err());
    }
}
