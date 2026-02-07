# Open Clanker Configuration Guide

## Overview

Open Clanker uses a single TOML configuration file with environment variable overrides for sensitive values.

## Configuration File

Location: `/etc/open-clanker/config.toml` (inside container) or `config.toml` (local)

## Example Configuration

```toml
# Server Configuration
[server]
host = "0.0.0.0"
port = 18789

# Channel Configuration
[channels.telegram]
bot_token = "your-telegram-bot-token"
allowed_chats = []  # Empty means all chats allowed

[channels.discord]
bot_token = "your-discord-bot-token"
guild_id = null  # Optional: restrict to specific guild

# Agent Configuration
[agent]
provider = "anthropic"  # Options: anthropic, openai, groq
model = "claude-sonnet-4-20250514"
api_key_env = "OPENCLAW_ANTHROPIC_API_KEY"
max_tokens = 4096

# Optional: Custom API base URL (for Groq, testing, etc.)
# api_base_url = "https://api.groq.com/openai/v1"

# Logging Configuration
[logging]
level = "info"  # Options: error, warn, info, debug, trace
format = "json"  # Options: json, pretty
```

## Configuration Options

### Server Section

```toml
[server]
host = "0.0.0.0"           # Bind address (0.0.0.0 = all interfaces)
port = 18789                  # Port to listen on
```

### Channels Section

#### Telegram

```toml
[channels.telegram]
bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
allowed_chats = ["-1001234567890"]  # Optional: restrict to specific chats
```

**Getting Telegram Bot Token:**
1. Message [@BotFather](https://t.me/BotFather) on Telegram
2. Send `/newbot`
3. Follow the prompts
4. Copy the token provided

**Getting Chat ID:**
1. Send a message to your bot
2. Visit: `https://api.telegram.org/bot<TOKEN>/getUpdates`
3. Find the `chat.id` in the response

#### Discord

```toml
[channels.discord]
bot_token = "MTIzNDU2Nzg5MA.GhI-jK.abcdefghijklmnopqrstuvwxyz123456"
guild_id = "123456789012345678"  # Optional: restrict to specific guild
```

**Getting Discord Bot Token:**
1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Go to "Bot" → "Add Bot"
4. Copy the token

**Getting Guild ID:**
1. Enable Developer Mode in Discord
2. Right-click on server → "Copy ID"

### Agent Section

#### Anthropic

```toml
[agent]
provider = "anthropic"
model = "claude-sonnet-4-20250514"  # See models below
api_key_env = "OPENCLAW_ANTHROPIC_API_KEY"
max_tokens = 4096
```

**Anthropic Models:**
- `claude-sonnet-4-20250514` - Sonnet 4 (balanced)
- `claude-opus-4-20250514` - Opus 4 (most capable)
- `claude-haiku-4-20250514` - Haiku 4 (fastest)
- `claude-3-5-sonnet-20241022` - Claude 3.5 Sonnet
- `claude-3-opus-20240229` - Claude 3 Opus

**API Key:**
Get from [Anthropic Console](https://console.anthropic.com/)

**Pricing:**
- Sonnet 4: $3/input M tokens, $15/output M tokens
- Opus 4: $15/input M tokens, $75/output M tokens
- Haiku 4: $0.80/input M tokens, $4/output M tokens

#### OpenAI

```toml
[agent]
provider = "openai"
model = "gpt-4-turbo"  # See models below
api_key_env = "OPENCLAW_OPENAI_API_KEY"
max_tokens = 4096
```

**OpenAI Models:**
- `gpt-4-turbo` - GPT-4 Turbo
- `gpt-4-turbo-preview` - GPT-4 Turbo Preview
- `gpt-4` - GPT-4
- `gpt-3.5-turbo` - GPT-3.5 Turbo (faster, cheaper)

**API Key:**
Get from [OpenAI Platform](https://platform.openai.com/api-keys)

**Pricing:**
- GPT-4 Turbo: $10/input M tokens, $30/output M tokens
- GPT-4: $30/input M tokens, $60/output M tokens
- GPT-3.5 Turbo: $0.50/input M tokens, $1.50/output M tokens

#### Groq (Ultra-Fast)

```toml
[agent]
provider = "groq"
model = "llama-3.3-70b-versatile"  # See models below
api_key_env = "OPENCLAW_GROQ_API_KEY"
max_tokens = 4096
# Optional: Custom API base URL
# api_base_url = "https://api.groq.com/openai/v1"
```

**Groq Models:**
- `llama-3.3-70b-versatile` - Llama 3.3 70B (best overall)
- `llama-3.1-70b-versatile` - Llama 3.1 70B (very fast)
- `mixtral-8x7b-32768` - Mixtral 8x7B (32k context)
- `gemma2-9b-it` - Gemma 2 9B (extremely fast)

**API Key:**
Get from [Groq Console](https://console.groq.com/)

**Pricing:**
- Llama 3.3 70B: $0.59/input M tokens, $0.59/output M tokens
- Llama 3.1 70B: $0.59/input M tokens, $0.59/output M tokens
- Mixtral 8x7B: $0.27/input M tokens, $0.27/output M tokens
- Gemma 2 9B: $0.08/input M tokens, $0.08/output M tokens

**Performance:**
- 15-30x faster than OpenAI
- 50x cheaper than GPT-4

### Logging Section

```toml
[logging]
level = "info"      # Logging level
format = "json"     # Log format
```

**Logging Levels:**
- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Informational messages (default)
- `debug` - Debug information
- `trace` - Trace-level detail (verbose)

**Log Formats:**
- `json` - Structured JSON logs (default)
- `pretty` - Human-readable plain text

## Environment Variables

Sensitive values (API keys, bot tokens) should be set via environment variables and can override config values.

### Required Environment Variables

Based on your `provider` choice:

#### For Anthropic:
```bash
OPENCLAW_ANTHROPIC_API_KEY=sk-ant-your-key-here
```

#### For OpenAI:
```bash
OPENCLAW_OPENAI_API_KEY=sk-openai-your-key-here
```

#### For Groq:
```bash
OPENCLAW_GROQ_API_KEY=gsk_your-key-here
```

### Channel Environment Variables

These override the config file values:

```bash
# Telegram
OPENCLAW_TELEGRAM_BOT_TOKEN=your-telegram-bot-token

# Discord
OPENCLAW_DISCORD_BOT_TOKEN=your-discord-bot-token
```

### Server Environment Variables

```bash
# Override host/port
OPENCLAW_HOST=0.0.0.0
OPENCLAW_PORT=18789

# Logging level
RUST_LOG=info  # Overrides config logging.level
```

## Configuration Validation

```bash
# Validate configuration file
open-clanker config-validate

# Validate specific config
open-clanker -c /path/to/config.toml config-validate
```

## Provider Comparison

| Provider | Best For | Speed | Cost | Context |
|----------|-----------|-------|------|--------|
| **Anthropic** | Complex reasoning, coding | Fast | $$ | 200k tokens |
| **OpenAI** | General purpose, coding | Fast | $$$ | 128k tokens |
| **Groq** | Fast responses, cost-sensitive | Very Fast | $ | 8k-128k tokens |

### When to Use Each Provider

**Use Anthropic (Claude) when:**
- You need strong reasoning capabilities
- Complex problem-solving
- Coding assistance
- Budget is not a constraint
- Need long context (200k tokens)

**Use OpenAI (GPT-4) when:**
- You need GPT-4 specific features
- Compatibility with existing workflows
- General purpose tasks
- Budget allows premium pricing

**Use Groq when:**
- Speed is critical (15-30x faster than OpenAI)
- Cost is important (50x cheaper than GPT-4)
- Fast responses needed
- Testing/prototyping
- Simple to moderate complexity tasks

## Advanced Configuration

### Multiple Agents (Future)

```toml
# Future: Support multiple agents
[agents.primary]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[agents.fast]
provider = "groq"
model = "llama-3.3-70b-versatile"

[agents.coding]
provider = "openai"
model = "gpt-4-turbo"
```

### Rate Limiting (Future)

```toml
[agent]
provider = "anthropic"
max_tokens = 4096

[rate_limits]
requests_per_minute = 60
tokens_per_minute = 100000
```

### Retry Configuration (Future)

```toml
[agent]
provider = "anthropic"

[retry]
max_attempts = 3
initial_delay_ms = 100
max_delay_ms = 10000
backoff_multiplier = 2
```

## Troubleshooting

### Configuration Not Found

```
Error: Configuration file not found
```

**Solution:**
```bash
# Generate default config
open-clanker config-generate

# Specify config path
open-clanker -c /path/to/config.toml gateway
```

### Invalid API Key

```
Error: Authentication failed
```

**Solution:**
1. Verify API key is correct
2. Check environment variable: `echo $OPENCLAW_ANTHROPIC_API_KEY`
3. Ensure no extra spaces or quotes

### Bot Token Invalid

```
Error: Channel error: Unauthorized
```

**Solution:**
1. Verify bot token is correct
2. Ensure bot has necessary permissions
3. Check bot is not disabled

### Connection Refused

```
Error: Could not connect to agent API
```

**Solution:**
1. Check internet connectivity
2. Verify API endpoint is reachable
3. Check firewall rules

### Rate Limit Exceeded

```
Error: Rate limit exceeded
```

**Solution:**
1. Wait before retrying
2. Consider switching to Groq (higher limits)
3. Implement exponential backoff in application

## Best Practices

### Security

1. **Never commit API keys** to version control
2. **Use environment variables** for sensitive data
3. **Rotate API keys** regularly
4. **Use read-only permissions** where possible
5. **Monitor usage** for unusual activity

### Performance

1. **Choose appropriate model** for your use case
2. **Set reasonable max_tokens** to reduce cost
3. **Use Groq for speed-critical tasks**
4. **Enable caching** when available (future feature)
5. **Monitor latency** and adjust model accordingly

### Cost Management

1. **Track token usage** with logging
2. **Use cheaper models** for simple tasks
3. **Set token limits** to prevent overages
4. **Consider Groq** for cost-sensitive applications
5. **Review provider pricing** regularly

## Configuration Templates

### Template 1: Development (Free)

```toml
[server]
host = "127.0.0.1"
port = 18789

[channels.telegram]
bot_token = "your-telegram-bot-token"

[agent]
provider = "groq"
model = "gemma2-9b-it"
api_key_env = "OPENCLAW_GROQ_API_KEY"
max_tokens = 2048

[logging]
level = "debug"
format = "pretty"
```

### Template 2: Production (Anthropic)

```toml
[server]
host = "0.0.0.0"
port = 18789

[channels.telegram]
bot_token = "your-telegram-bot-token"
allowed_chats = ["-1001234567890"]

[agent]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "OPENCLAW_ANTHROPIC_API_KEY"
max_tokens = 4096

[logging]
level = "info"
format = "json"
```

### Template 3: High Performance (Groq)

```toml
[server]
host = "0.0.0.0"
port = 18789

[channels.telegram]
bot_token = "your-telegram-bot-token"
allowed_chats = ["-1001234567890"]

[channels.discord]
bot_token = "your-discord-bot-token"

[agent]
provider = "groq"
model = "llama-3.3-70b-versatile"
api_key_env = "OPENCLAW_GROQ_API_KEY"
max_tokens = 4096

[logging]
level = "info"
format = "json"
```

## Quick Reference

| Option | Values | Default |
|--------|--------|---------|
| `provider` | `anthropic`, `openai`, `groq` | `anthropic` |
| `host` | IP address or hostname | `0.0.0.0` |
| `port` | 1-65535 | `18789` |
| `max_tokens` | 1-128000 (varies by model) | `4096` |
| `log_level` | `error`, `warn`, `info`, `debug`, `trace` | `info` |
| `log_format` | `json`, `pretty` | `json` |

## Resources

- [Anthropic Documentation](https://docs.anthropic.com/)
- [OpenAI Documentation](https://platform.openai.com/docs)
- [Groq Documentation](https://console.groq.com/docs)
- [Telegram Bot API](https://core.telegram.org/bots/api)
- [Discord Developer Portal](https://discord.com/developers/docs)

## Support

For configuration issues:
1. Check logs: `docker logs open-clanker`
2. Validate config: `open-clanker config-validate`
3. Enable debug logging: `RUST_LOG=debug`
4. Review this guide for common issues

Questions? Open an issue on GitHub.
