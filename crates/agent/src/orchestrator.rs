//! Master_Clanker orchestration: coordinates Worker_Clankers for subtasks.
//!
//! Master_Clanker uses the primary agent (Claude, OpenAI, Grok) to decide when to delegate.
//! Worker_Clankers use Groq and execute tasks with assigned identities.

use crate::factory::AgentFactory;
use crate::types::{Agent, AgentMessage, AgentResponse, MessageRole, WorkerResult, WorkerTask};
use clanker_config::{AgentConfig, WorkerAgentConfig};
use std::sync::Arc;
use tracing::{debug, error, warn};

/// Delegation marker in Master response - when present, we parse and spawn workers
const DELEGATE_PREFIX: &str = "[DELEGATE]";

/// Master_Clanker system prompt for orchestration
pub const MASTER_SYSTEM_PROMPT: &str = r#"You are Master_Clanker, an orchestration agent that coordinates Worker_Clankers for complex tasks.

When you need to delegate to workers, your response MUST start with [DELEGATE] followed by a JSON array of worker assignments. Each assignment has "identity" (e.g. "Research Assistant", "Code Reviewer") and "task" (the specific subtask). Example:

[DELEGATE][{"identity":"Research Assistant","task":"Find recent studies on topic X"},{"identity":"Summarizer","task":"Synthesize the findings"}]

You may spawn up to 5 workers. Each worker gets a distinct identity and a specific task.

If you can answer the user's question directly without delegation, respond normally. Do NOT use [DELEGATE] for simple queries."#;

/// Orchestrator that wraps the Master agent and spawns Worker_Clankers
pub struct MasterClanker {
    master_agent: Arc<dyn Agent + Send + Sync>,
    worker_config: WorkerAgentConfig,
    max_workers: usize,
}

impl MasterClanker {
    /// Create a new MasterClanker
    pub fn new(
        master_agent: Arc<dyn Agent + Send + Sync>,
        worker_config: WorkerAgentConfig,
        max_workers: usize,
    ) -> Self {
        Self {
            master_agent,
            worker_config,
            max_workers,
        }
    }

    /// Spawn a single Worker_Clanker with identity and task
    pub async fn spawn_worker(
        &self,
        identity: &str,
        task: &str,
    ) -> Result<AgentResponse, crate::types::AgentError> {
        let groq_config = self.worker_config_to_agent_config();
        if groq_config.api_key.is_none() || groq_config.api_key.as_ref().unwrap().is_empty() {
            return Err(crate::types::AgentError::AuthenticationFailed);
        }

        let worker = AgentFactory::create_arc_from_config(groq_config);
        let system_prompt = format!(
            "You are Worker_Clanker. Your identity: {}. Execute this task: {}",
            identity, task
        );

        let messages = vec![
            AgentMessage {
                role: MessageRole::System,
                content: system_prompt,
            },
            AgentMessage {
                role: MessageRole::User,
                content: task.to_string(),
            },
        ];

        debug!("Spawning Worker_Clanker: identity={}, task_len={}", identity, task.len());
        worker.chat(messages).await
    }

    /// Delegate tasks to Worker_Clankers in parallel (up to max_workers)
    pub async fn delegate(&self, workers: Vec<WorkerTask>) -> Vec<WorkerResult> {
        let workers: Vec<WorkerTask> = workers.into_iter().take(self.max_workers).collect();

        let mut handles = Vec::with_capacity(workers.len());
        for wt in workers {
            let identity = wt.identity.clone();
            let task = wt.task.clone();
            let worker_config = self.worker_config.clone();
            handles.push(tokio::spawn(async move {
                let groq_config = worker_config_to_agent_config(&worker_config);
                if groq_config.api_key.is_none() || groq_config.api_key.as_ref().unwrap().is_empty() {
                    warn!("Worker_Clanker: Groq API key not set, skipping");
                    return WorkerResult {
                        identity: identity.clone(),
                        task: task.clone(),
                        content: format!("[Error: Groq API key not configured for worker {}]", identity),
                    };
                }

                let worker = AgentFactory::create_arc_from_config(groq_config);
                let system_prompt = format!(
                    "You are Worker_Clanker. Your identity: {}. Execute this task: {}",
                    identity, task
                );

                let messages = vec![
                    AgentMessage {
                        role: MessageRole::System,
                        content: system_prompt,
                    },
                    AgentMessage {
                        role: MessageRole::User,
                        content: task.clone(),
                    },
                ];

                match worker.chat(messages).await {
                    Ok(resp) => WorkerResult {
                        identity,
                        task,
                        content: resp.content,
                    },
                    Err(e) => {
                        error!("Worker_Clanker {} failed: {}", identity, e);
                        WorkerResult {
                            identity,
                            task,
                            content: format!("[Worker error: {}]", e),
                        }
                    }
                }
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for h in handles {
            match h.await {
                Ok(r) => results.push(r),
                Err(e) => {
                    error!("Worker join error: {}", e);
                }
            }
        }
        results
    }

    /// Get the master agent for direct chat
    pub fn master_agent(&self) -> Arc<dyn Agent + Send + Sync> {
        self.master_agent.clone()
    }

    /// Parse delegation from Master response; returns None if no delegation
    pub fn parse_delegation(response: &str) -> Option<Vec<WorkerTask>> {
        let trimmed = response.trim();
        if !trimmed.starts_with(DELEGATE_PREFIX) {
            return None;
        }

        let json_start = trimmed
            .strip_prefix(DELEGATE_PREFIX)
            .map(|s| s.trim())
            .unwrap_or("");
        if json_start.is_empty() {
            return None;
        }

        // Find the JSON array - it may be followed by more text
        let json_str = extract_json_array(json_start)?;
        let tasks: Vec<WorkerTask> = serde_json::from_str(&json_str).ok()?;
        if tasks.is_empty() {
            return None;
        }
        Some(tasks)
    }

    fn worker_config_to_agent_config(&self) -> AgentConfig {
        worker_config_to_agent_config(&self.worker_config)
    }
}

fn worker_config_to_agent_config(worker: &WorkerAgentConfig) -> AgentConfig {
    AgentConfig {
        provider: "groq".to_string(),
        model: worker.model.clone(),
        api_key_env: worker.api_key_env.clone(),
        api_key: worker.api_key.clone(),
        max_tokens: worker.max_tokens,
        api_base_url: None,
        worker: None,
    }
}

/// Extract the first complete JSON array from the string
fn extract_json_array(s: &str) -> Option<String> {
    let s = s.trim();
    if !s.starts_with('[') {
        return None;
    }
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut quote_char = '"';
    for (i, c) in s.chars().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            if c == '\\' {
                escape = true;
            } else if c == quote_char {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                in_string = true;
                quote_char = c;
            }
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(s[..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_delegation_empty() {
        assert!(MasterClanker::parse_delegation("Hello world").is_none());
        assert!(MasterClanker::parse_delegation("").is_none());
    }

    #[test]
    fn test_parse_delegation_valid() {
        let s = r#"[DELEGATE][{"identity":"Research Assistant","task":"Find studies"}]"#;
        let tasks = MasterClanker::parse_delegation(s).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].identity, "Research Assistant");
        assert_eq!(tasks[0].task, "Find studies");
    }

    #[test]
    fn test_parse_delegation_multiple() {
        let s = r#"[DELEGATE][{"identity":"A","task":"T1"},{"identity":"B","task":"T2"}]"#;
        let tasks = MasterClanker::parse_delegation(s).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].identity, "A");
        assert_eq!(tasks[1].identity, "B");
    }

    #[test]
    fn test_parse_delegation_with_trailing_text() {
        let s = r#"[DELEGATE][{"identity":"X","task":"Y"}] and some extra text"#;
        let tasks = MasterClanker::parse_delegation(s).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].identity, "X");
    }

    #[test]
    fn test_extract_json_array() {
        assert_eq!(
            extract_json_array(r#"[{"a":1}]"#),
            Some(r#"[{"a":1}]"#.to_string())
        );
        assert_eq!(
            extract_json_array(r#"[{"a":"b"}]"#),
            Some(r#"[{"a":"b"}]"#.to_string())
        );
    }

    #[tokio::test]
    async fn test_delegate_respects_max_workers() {
        use clanker_config::WorkerAgentConfig;

        let master = AgentFactory::create_arc_from_config(AgentConfig {
            provider: "placeholder".to_string(),
            model: "test".to_string(),
            api_key_env: "TEST".to_string(),
            api_key: None,
            max_tokens: 100,
            api_base_url: None,
            worker: None,
        });

        let worker_config = WorkerAgentConfig {
            model: "test".to_string(),
            api_key_env: "GROQ_TEST".to_string(),
            api_key: None,
            max_tokens: 100,
        };

        let orchestrator = MasterClanker::new(master, worker_config, 2);

        let workers = vec![
            WorkerTask {
                identity: "A".to_string(),
                task: "T1".to_string(),
            },
            WorkerTask {
                identity: "B".to_string(),
                task: "T2".to_string(),
            },
            WorkerTask {
                identity: "C".to_string(),
                task: "T3".to_string(),
            },
        ];

        let results = orchestrator.delegate(workers).await;

        assert_eq!(results.len(), 2, "delegate should cap at max_workers=2");
    }
}
