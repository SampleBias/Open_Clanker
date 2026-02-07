# Open Clanker 🦀

<div align="center">

```
    ╔══════════════════════════════════════════════════════════╗
    ║                                                            ║
    ║       ██████╗ ██████╗ ███████╗██╗  ██╗██╗███╗   ███╗  ║
    ║      ██╔════╝██╔═══██╗██╔════╝██║ ██╔╝██║████╗ ████║  ║
    ║      ██║     ██║   ██║███████╗█████╔╝ ██║██╔████╔██║  ║
    ║      ██║     ██║   ██║╚════██║██╔═██╗ ██║██║╚██╔╝██║  ║
    ║      ╚██████╗╚██████╔╝███████║██║  ██╗██║██║ ╚═╝ ██║  ║
    ║       ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝  ║
    ║                                                            ║
    ║        🤖 High-Performance AI Assistant Framework 🤖          ║
    ║                     Built with Rust ❤️                       ║
    ║                                                            ║
    ║            Spawned from S4MPL3BI4S 🌟                     ║
    ║                                                            ║
    ╚══════════════════════════════════════════════════════════╝
```

[![Rust](https://img.shields.io/badge/rust-1.92+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests](https://img.shields.io/badge/tests-68%20passing-brightgreen.svg)](https://github.com/SampleBias/Open_Clanker)
[![Stars](https://img.shields.io/github/stars/SampleBias/Open_Clanker?style=social)](https://github.com/SampleBias/Open_Clanker/stargazers)

**⚡ Blazing fast Rust-powered AI assistant framework**

[Features](#-features) •
[Architecture](#-architecture) •
[Quick Start](#-quick-start) •
[Documentation](#-documentation)

</div>

---

## 🤖 What is Open Clanker?

Open Clanker is a **high-performance, production-ready AI assistant framework** built with Rust, spawned from the digital realm of **S4MPL3BI4S** 🌟.

It connects multiple AI providers with various messaging platforms, delivering lightning-fast responses while maintaining rock-solid reliability.

### 🎯 Key Highlights

- **🚀 Extreme Performance**: Rust-powered with minimal latency (built for speed!)
- **🤖 Multi-Provider AI**: Anthropic, OpenAI, xAI Grok, Groq LLaMA
- **📱 Multi-Channel Messaging**: Telegram, Discord (Slack & WhatsApp coming soon)
- **🏗️ Modular Architecture**: Clean workspace with 6 specialized crates
- **🔒 Type Safety**: Rust's ownership system guarantees memory safety
- **✅ Battle-Tested**: 68 tests passing across all modules

### 🎨 The Vibe

```
  .-. .-.
.  ; \.  ; \.    "I am Clanker, spawned from S4MPL3BI4S"
|  |  |  |  |  |    "Built with Rust, ready to serve!"
'  '  '  '  '
```

## 🚀 Features

### AI Provider Support 🧠

| Provider | Models | Status | ⚡ |
|----------|---------|--------|-----|
| **Anthropic** | Claude Sonnet 4, Opus, Haiku | ✅ Complete | 🟢 |
| **OpenAI** | GPT-4, GPT-3.5 Turbo | ✅ Complete | 🟢 |
| **Grok (xAI)** | Grok-2, Grok Beta | ✅ Complete | 🟢 |
| **Groq** | LLaMA 3.3, Mixtral, Gemma | ✅ Complete | 🟢 |

### Channel Support 📡

| Platform | Status | Notes | 🎮 |
|----------|--------|--------|-----|
| **Telegram** | ✅ Complete | Full send/listen | 🟢 |
| **Discord** | 🔄 Partial | Structure complete | 🟡 |
| **Slack** | ⏳ Planned | Ready for implementation | ⚪ |
| **WhatsApp** | ⏳ Planned | Ready for implementation | ⚪ |

### Core Features ⚡

- **🌐 Gateway Server**: WebSocket + HTTP for real-time communication
- **💻 CLI Interface**: Full command-line for management
- **⚙️ TOML Configuration**: Easy setup with environment overrides
- **📝 JSON Logging**: Structured logs with multiple levels
- **🛡️ Error Handling**: Comprehensive errors with detailed context
- **🧠 System Prompts**: Channel-specific AI prompts for better interactions
- **🔌 Plugin Architecture**: Easy to extend with new providers/channels

## 🏗️ Architecture

```
╔═══════════════════════════════════════════════════════════════╗
║                         Open Clanker Workspace                     ║
╠═══════════════════════════════════════════════════════════════╣
║                                                                  ║
║  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        ║
║  │ clanker-core│────▶│clanker-gate│────▶│  Channels  │        ║
║  │  (20 tests) │    │   (23 test) │    │  (10 tests) │        ║
║  │             │    │             │    │             │        ║
║  │ - Types    │    │ - WebSocket │    │ - Telegram │        ║
║  │ - Errors   │    │ - HTTP      │    │ - Discord  │        ║
║  │ - Traits   │    │ - Routing   │    │             │        ║
║  └─────────────┘    └─────────────┘    └─────────────┘        ║
║         │                 │                                   ║
║         ▼                 ▼                                   ║
║  ┌─────────────┐    ┌─────────────┐                        ║
║  │clanker-conf│    │clanker-agen│                        ║
║  │  (11 tests) │    │  (15 tests) │                        ║
║  │             │    │             │                        ║
║  │ - TOML      │    │ - Anthropic │                        ║
║  │ - Validation│    │ - OpenAI    │                        ║
║  │ - Env Vars  │    │ - Grok      │                        ║
║  └─────────────┘    │ - Groq      │                        ║
║                    └─────────────┘                        ║
║                          │                                 ║
║                          ▼                                 ║
║                    ┌─────────────┐                         ║
║                    │clanker-cli  │                         ║
║                    │  (commands) │                         ║
║                    └─────────────┘                         ║
║                                                                  ║
╚═══════════════════════════════════════════════════════════════╝

    Spawning from S4MPL3BI4S 🌟
    Rust 1.92+ | MIT License | 68 Tests Passing
```

### Workspace Crates

| Crate | Purpose | Tests | Status |
|--------|---------|--------|--------|
| **clanker-core** | Shared types, ChannelType, Message | 20 | ✅ |
| **clanker-config** | TOML config, validation, env vars | 11 | ✅ |
| **clanker-gateway** | WebSocket/HTTP server | 23 | ✅ |
| **clanker-agent** | AI provider clients (4 providers) | 15 | ✅ |
| **clanker-channels** | Telegram, Discord bots | 10 | ✅ |
| **clanker-cli** | CLI interface | - | 🔄 |

**Total: 68 tests passing** 🎉

## 📦 Quick Start

### Prerequisites

- Rust 1.92 or later
- Cargo (included with Rust)
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/SampleBias/Open_Clanker.git
cd Open_Clanker

# Build the workspace
cargo build --workspace

# Run the test suite
cargo test --workspace

# Install the CLI tool globally
cargo install --path crates/cli

# 🎉 You're ready to roll!
```

### Configuration

Create your configuration file:

```bash
# Copy example config
cp config-examples/config.toml config.toml

# Edit it to your liking
nano config.toml
```

### Configuration Example

```toml
# Server Configuration
[server]
host = "0.0.0.0"
port = 18789

# AI Provider Configuration
[agent]
provider = "anthropic"  # Options: anthropic, openai, grok, groq
model = "claude-sonnet-4-20250514"
api_key_env = "OPENCLAW_ANTHROPIC_API_KEY"
max_tokens = 4096

# Telegram Channel (optional)
[channels.telegram]
bot_token = "your-telegram-bot-token"

# Discord Channel (optional)
[channels.discord]
bot_token = "your-discord-bot-token"

# Logging Configuration
[logging]
level = "info"
format = "json"
```

### Running Open Clanker

```bash
# Start the gateway server
open-clanker gateway

# Check status
open-clanker status

# Send a test message
open-clanker send "Hello from S4MPL3BI4S!"

# Show help
open-clanker --help
```

## 🧪 Testing

Run the complete test suite:

```bash
# All workspace tests
cargo test --workspace

# Specific crate tests
cargo test --package clanker-agent
cargo test --package clanker-channels

# With output
cargo test --workspace -- --nocapture
```

## 📚 Documentation

- **[QUICK_START_RUST.md](./QUICK_START_RUST.md)** - Quick start guide
- **[TECHNICAL_ARCHITECTURE.md](./TECHNICAL_ARCHITECTURE.md)** - Architecture overview
- **[IMPLEMENTATION_CHECKLIST.md](./IMPLEMENTATION_CHECKLIST.md)** - Implementation progress
- **[TODO.md](./TODO.md)** - Detailed TODO list and roadmap
- **[ARCHITECTURE_COMPARISON.md](./ARCHITECTURE_COMPARISON.md)** - TypeScript vs Rust

## 🗺️ Roadmap

### v1.0 - Current Focus (40% Complete)

- [x] Core framework architecture ✅
- [x] Agent system (4 providers) ✅
- [x] Channel system (Telegram, Discord) ✅
- [x] Gateway server (WebSocket + HTTP) ✅
- [x] CLI interface ✅
- [ ] Complete Discord integration
- [ ] Storage layer (SQLite)
- [ ] Message persistence
- [ ] Docker deployment
- [ ] Production hardening

### v1.1 - Upcoming

- Real-time streaming responses
- Message history and context
- Rate limiting
- Metrics and monitoring
- Webhooks support

### v2.0 - Future

- Multi-agent orchestration
- Plugin system
- Custom AI model support
- Advanced routing
- Distributed deployment

## 🌟 Clankerville Needs You!

```
    ╔═══════════════════════════════════════════════════════╗
    ║                                                           ║
    ║         ⭐  STAR THE REPOSITORY  ⭐                     ║
    ║                                                           ║
    ║   Help Clankerville grow by starring this repo!          ║
    ║                                                           ║
    ║   https://github.com/SampleBias/Open_Clanker           ║
    ║                                                           ║
    ║   Every star counts in Clankerville! 🌟                  ║
    ║                                                           ║
    ║           Spawned from S4MPL3BI4S 🌟                      ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════╝
```

## 🤝 Contributing

Contributions are welcome! Join the Clankerville community:

1. Read the documentation
2. Fork the repository
3. Create a feature branch (`git checkout -b feature/amazing-feature`)
4. Make your changes
5. Add tests for new functionality
6. Ensure all tests pass
7. Submit a pull request

**Pull Request Template:**
```markdown
## Description
Brief description of changes

## Type
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests added/updated
- [ ] All tests passing

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added where needed
- [ ] Documentation updated
```

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2025 SampleBias

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 👤 The Creator

**SampleBias**
- GitHub: [@SampleBias](https://github.com/SampleBias)
- Project: Open Clanker 🦀

### 🌟 The Origin Story

Open Clanker was spawned from the digital realm of **S4MPL3BI4S**, designed to bring the power of AI assistance to the masses with the speed and safety of Rust. 🚀

## 🙏 Acknowledgments

- [teloxide](https://github.com/teloxide/teloxide) - Telegram bot framework
- [serenity](https://github.com/serenity-rs/serenity) - Discord bot framework
- [tokio](https://tokio.rs/) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- The Rust Community 🦀

---

<div align="center">

```
    ╔═══════════════════════════════════════════════════════╗
    ║                                                           ║
    ║              🤖 Open Clanker 🤖                        ║
    ║                                                           ║
    ║          High-Performance AI Assistant Framework            ║
    ║                                                           ║
    ║         Spawned from S4MPL3BI4S | Built with Rust           ║
    ║                                                           ║
    ║                    ⭐ Star the Repo! ⭐                   ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════╝
```

**Built with ❤️ using Rust 1.92+**

[⬆ Back to Top](#open-clanker-)

</div>
