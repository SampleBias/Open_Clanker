#[cfg(test)]
mod error_tests {
    use super::ClankerError;

    #[test]
    fn test_error_creation() {
        let err = ClankerError::Config("test error".to_string());
        assert_eq!(err.error_code(), "CONFIG_ERROR");

        let err = ClankerError::Channel {
            channel: "telegram".to_string(),
            message: "connection failed".to_string(),
        };
        assert_eq!(err.error_code(), "CHANNEL_ERROR");

        let err = ClankerError::Agent {
            provider: "anthropic".to_string(),
            message: "api error".to_string(),
        };
        assert_eq!(err.error_code(), "AGENT_ERROR");

        let err = ClankerError::NotFound("user".to_string());
        assert_eq!(err.error_code(), "NOT_FOUND_ERROR");

        let err = ClankerError::InvalidInput("bad input".to_string());
        assert_eq!(err.error_code(), "INVALID_INPUT_ERROR");
    }

    #[test]
    fn test_error_display() {
        let err = ClankerError::Config("missing field".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing field");

        let err = ClankerError::Channel {
            channel: "telegram".to_string(),
            message: "timeout".to_string(),
        };
        assert_eq!(err.to_string(), "Channel error [telegram]: timeout");

        let err = ClankerError::Agent {
            provider: "anthropic".to_string(),
            message: "rate limit".to_string(),
        };
        assert_eq!(err.to_string(), "Agent error [anthropic]: rate limit");
    }

    #[test]
    fn test_retryable_errors() {
        // Test IO error - should be retryable
        let io_err = std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "connection reset",
        );
        let err = ClankerError::Io(io_err);
        assert!(err.is_retryable(), "IO error should be retryable");

        // Test Timeout - should be retryable
        let err = ClankerError::Timeout;
        assert!(err.is_retryable(), "Timeout should be retryable");

        // Test Rate Limit - should be retryable
        let err = ClankerError::RateLimit;
        assert!(err.is_retryable(), "Rate limit should be retryable");

        // Test Agent with rate limit message - should be retryable
        let err = ClankerError::Agent {
            provider: "anthropic".to_string(),
            message: "rate limit exceeded".to_string(),
        };
        assert!(err.is_retryable(), "Agent rate limit should be retryable");

        // Test Agent with invalid API key - should NOT be retryable
        let err = ClankerError::Agent {
            provider: "anthropic".to_string(),
            message: "invalid api key".to_string(),
        };
        assert!(!err.is_retryable(), "Agent invalid API key should not be retryable");

        // Test Config error - should NOT be retryable
        let err = ClankerError::Config("invalid".to_string());
        assert!(!err.is_retryable(), "Config error should not be retryable");
    }
}
