use thiserror::Error;

/// Main error type for Open Clanker
#[derive(Error, Debug)]
pub enum ClankerError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Configuration file errors
    #[error("Configuration file error: {0}")]
    ConfigFile(String),

    /// Channel-specific errors
    #[error("Channel error [{channel}]: {message}")]
    Channel { channel: String, message: String },

    /// AI agent errors
    #[error("Agent error [{provider}]: {message}")]
    Agent { provider: String, message: String },

    /// Network/HTTP errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing errors
    #[error("TOML parsing error: {0}")]
    TomlParse(String),

    /// Authentication failures
    #[error("Authentication failed")]
    Authentication,

    /// Rate limiting
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Invalid user input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Operation timeout
    #[error("Operation timed out")]
    Timeout,

    /// Bot errors (Telegram/Discord)
    #[error("Bot error: {0}")]
    Bot(String),

    /// Environment variable errors
    #[error("Environment variable error: {0}")]
    Environment(String),
}

/// Type alias for Result with ClankerError
pub type Result<T> = std::result::Result<T, ClankerError>;

impl ClankerError {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a configuration file error
    pub fn config_file(msg: impl Into<String>) -> Self {
        Self::ConfigFile(msg.into())
    }

    /// Create a channel error
    pub fn channel(channel: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Channel {
            channel: channel.into(),
            message: message.into(),
        }
    }

    /// Create an agent error
    pub fn agent(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Agent {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(item: impl Into<String>) -> Self {
        Self::NotFound(item.into())
    }

    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) |
            Self::Io(_) |  // IO errors can be retryable (transient)
            Self::Timeout |
            Self::RateLimit |
            Self::Agent { provider: _, message: _ } if self.is_transient_agent_error()
        )
    }

    /// Check if agent error is transient (retryable)
    fn is_transient_agent_error(&self) -> bool {
        if let Self::Agent { message, .. } = self {
            let msg_lower = message.to_lowercase();
            msg_lower.contains("timeout") ||
            msg_lower.contains("rate limit") ||
            msg_lower.contains("service unavailable") ||
            msg_lower.contains("temporary") ||
            msg_lower.contains("try again")
        } else {
            false
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG_ERROR",
            Self::ConfigFile(_) => "CONFIG_FILE_ERROR",
            Self::Channel { .. } => "CHANNEL_ERROR",
            Self::Agent { .. } => "AGENT_ERROR",
            Self::Network(_) => "NETWORK_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Serialization(_) => "SERIALIZATION_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::TomlParse(_) => "TOML_PARSE_ERROR",
            Self::Authentication => "AUTHENTICATION_ERROR",
            Self::RateLimit => "RATE_LIMIT_ERROR",
            Self::InvalidInput(_) => "INVALID_INPUT_ERROR",
            Self::NotFound(_) => "NOT_FOUND_ERROR",
            Self::Timeout => "TIMEOUT_ERROR",
            Self::Bot(_) => "BOT_ERROR",
            Self::Environment(_) => "ENVIRONMENT_ERROR",
        }
    }
}
