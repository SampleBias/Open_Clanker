use thiserror::Error;

/// Channel errors
#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Unsupported channel type: {0}")]
    UnsupportedChannel(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Listen error: {0}")]
    ListenError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Rate limited: retry after {0:?}")]
    RateLimited(Option<std::time::Duration>),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Message too long: {0} characters, max {1}")]
    MessageTooLong(usize, usize),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Channel result type
pub type Result<T> = std::result::Result<T, ChannelError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ChannelError::AuthenticationFailed;
        assert_eq!(err.to_string(), "Authentication failed");

        let err = ChannelError::SendFailed("Network error".to_string());
        assert!(err.to_string().contains("Send failed"));
    }
}
