use clanker_core::{ClankerError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure for Open Clanker
#[derive(Debug, Deserialize, Serialize)]
#[derive(Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub channels: ChannelsConfig,
    pub agent: AgentConfig,
    pub logging: LoggingConfig,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ClankerError::config_file(format!("Failed to read config file: {}", e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            ClankerError::TomlParse(format!("Failed to parse TOML: {}", e))
        })?;

        Ok(config)
    }

    /// Load configuration from environment variables
    /// Overrides values in the loaded config
    pub fn load_env(&mut self) -> Result<()> {
        // Load Telegram bot token
        if let Some(telegram) = &mut self.channels.telegram {
            if let Ok(token) = std::env::var("OPENCLAW_TELEGRAM_BOT_TOKEN") {
                telegram.bot_token = token;
            }
        }

        // Load Discord bot token
        if let Some(discord) = &mut self.channels.discord {
            if let Ok(token) = std::env::var("OPENCLAW_DISCORD_BOT_TOKEN") {
                discord.bot_token = token;
            }
        }

        // Load agent API key based on provider
        if let Ok(api_key) = std::env::var(&self.agent.api_key_env) {
            self.agent.api_key = Some(api_key);
        }

        // Override server config from environment
        if let Ok(host) = std::env::var("OPENCLAW_HOST") {
            self.server.host = host;
        }

        if let Ok(port) = std::env::var("OPENCLAW_PORT") {
            self.server.port = port.parse().map_err(|e| {
                ClankerError::Environment(format!("Invalid OPENCLAW_PORT: {}", e))
            })?;
        }

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 || self.server.port > 65535 {
            return Err(ClankerError::Config(format!(
                "Invalid port: {}. Must be between 1 and 65535",
                self.server.port
            )));
        }

        // Validate at least one channel is configured
        if self.channels.telegram.is_none() && self.channels.discord.is_none() {
            return Err(ClankerError::Config(
                "At least one channel (telegram or discord) must be configured".to_string(),
            ));
        }

        // Validate channel configurations
        if let Some(telegram) = &self.channels.telegram {
            if telegram.bot_token.is_empty() {
                return Err(ClankerError::Config(
                    "Telegram bot token cannot be empty".to_string(),
                ));
            }
        }

        if let Some(discord) = &self.channels.discord {
            if discord.bot_token.is_empty() {
                return Err(ClankerError::Config(
                    "Discord bot token cannot be empty".to_string(),
                ));
            }
        }

        // Validate agent configuration
        let valid_providers = ["anthropic", "openai", "grok", "groq"];
        if !valid_providers.contains(&self.agent.provider.as_str()) {
            return Err(ClankerError::Config(format!(
                "Invalid provider: {}. Must be one of: {:?}",
                self.agent.provider, valid_providers
            )));
        }

        if self.agent.model.is_empty() {
            return Err(ClankerError::Config(
                "Agent model cannot be empty".to_string(),
            ));
        }

        // Validate that API key is set
        if self.agent.api_key.is_none() || self.agent.api_key.as_ref().unwrap().is_empty() {
            return Err(ClankerError::Config(
                format!(
                    "Agent API key must be set via environment variable: {}",
                    self.agent.api_key_env
                )
                ));
        }

        // Validate logging config
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ClankerError::Config(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.logging.level, valid_log_levels
            )));
        }

        let valid_log_formats = ["json", "pretty"];
        if !valid_log_formats.contains(&self.logging.format.as_str()) {
            return Err(ClankerError::Config(format!(
                "Invalid log format: {}. Must be one of: {:?}",
                self.logging.format, valid_log_formats
            )));
        }

        Ok(())
    }

    /// Save configuration to a file
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            ClankerError::TomlParse(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            ClankerError::config_file(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }
}

/// Server configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls: Option<TlsConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 18789,
            tls: None,
        }
    }
}

/// TLS configuration for HTTPS
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

/// Channels configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChannelsConfig {
    pub telegram: Option<TelegramConfig>,
    pub discord: Option<DiscordConfig>,
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            telegram: Some(TelegramConfig::default()),
            discord: Some(DiscordConfig::default()),
        }
    }
}

/// Telegram bot configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_chats: Option<Vec<String>>,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: "your-telegram-bot-token".to_string(),
            allowed_chats: None,
        }
    }
}

/// Discord bot configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub guild_id: Option<String>,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            bot_token: "your-discord-bot-token".to_string(),
            guild_id: None,
        }
    }
}

/// AI agent configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,  // Loaded from environment, not saved to file
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,  // Optional: Custom API endpoint
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key_env: "OPENCLAW_ANTHROPIC_API_KEY".to_string(),
            api_key: None,
            max_tokens: 4096,
            api_base_url: None,
        }
    }
}

/// Logging configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

/// Generate a default configuration
pub fn generate_default_config() -> String {
    let config = Config {
        server: ServerConfig::default(),
        channels: ChannelsConfig::default(),
        agent: AgentConfig::default(),
        logging: LoggingConfig::default(),
    };

    toml::to_string_pretty(&config).unwrap_or_else(|e| {
        format!("Error generating config: {}", e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_generation() {
        let config_str = generate_default_config();

        assert!(config_str.contains("[server]"));
        assert!(config_str.contains("host = \"0.0.0.0\""));
        assert!(config_str.contains("port = 18789"));

        assert!(config_str.contains("[channels.telegram]"));
        assert!(config_str.contains("bot_token = \"your-telegram-bot-token\""));

        assert!(config_str.contains("[agent]"));
        assert!(config_str.contains("provider = \"anthropic\""));
        assert!(config_str.contains("model = \"claude-sonnet-4-20250514\""));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            server: ServerConfig::default(),
            channels: ChannelsConfig::default(),
            agent: AgentConfig::default(),
            logging: LoggingConfig::default(),
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.server.host, deserialized.server.host);
        assert_eq!(config.server.port, deserialized.server.port);
        assert_eq!(config.agent.provider, deserialized.agent.provider);
        assert_eq!(config.agent.model, deserialized.agent.model);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 18789,
                tls: None,
            },
            channels: ChannelsConfig {
                telegram: Some(TelegramConfig {
                    bot_token: "test-token".to_string(),
                    allowed_chats: None,
                }),
                discord: None,
            },
            agent: AgentConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                api_key_env: "OPENCLAW_ANTHROPIC_API_KEY".to_string(),
                api_key: Some("test-key".to_string()),
                max_tokens: 4096,
                api_base_url: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_port() {
        let mut config = Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 0,  // Invalid port (0 is invalid)  // Invalid - should compile time error  // Invalid port
                tls: None,
            },
            channels: ChannelsConfig::default(),
            agent: AgentConfig::default(),
            logging: LoggingConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_no_channel() {
        let config = Config {
            server: ServerConfig::default(),
            channels: ChannelsConfig {
                telegram: None,
                discord: None,
            },
            agent: AgentConfig {
                api_key: Some("test".to_string()),
                ..Default::default()
            },
            logging: LoggingConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_provider() {
        let config = Config {
            server: ServerConfig::default(),
            channels: ChannelsConfig::default(),
            agent: AgentConfig {
                provider: "invalid".to_string(),
                api_key: Some("test".to_string()),
                ..Default::default()
            },
            logging: LoggingConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_load_from_file() {
        let config_str = generate_default_config();

        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_str).unwrap();

        let config = Config::load_from_path(temp_file.path()).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 18789);
    }

    #[test]
    fn test_config_save_to_file() {
        let config = Config {
            server: ServerConfig::default(),
            channels: ChannelsConfig::default(),
            agent: AgentConfig::default(),
            logging: LoggingConfig::default(),
        };

        let temp_file = NamedTempFile::new().unwrap();
        config.save_to_path(temp_file.path()).unwrap();

        let loaded = Config::load_from_path(temp_file.path()).unwrap();
        assert_eq!(config.server.host, loaded.server.host);
    }

    #[test]
    fn test_config_defaults() {
        let server_config = ServerConfig::default();
        assert_eq!(server_config.host, "0.0.0.0");
        assert_eq!(server_config.port, 18789);

        let telegram_config = TelegramConfig::default();
        assert_eq!(telegram_config.bot_token, "your-telegram-bot-token");
        assert!(telegram_config.allowed_chats.is_none());

        let agent_config = AgentConfig::default();
        assert_eq!(agent_config.provider, "anthropic");
        assert_eq!(agent_config.model, "claude-sonnet-4-20250514");
        assert_eq!(agent_config.max_tokens, 4096);
    }
}
