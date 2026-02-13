# Open Clanker — Project Review

**Date**: 2026-02-13  
**Status**: Phase 3A — Gateway Integration (Agent + Channels wired)

---

## 🔄 Reset & Current State

### How to Run (fixes "command not found")

If you see `bash: command not found: open-clanker`:

```bash
# From project root — use the wrapper script:
./open-clanker

# Or via cargo:
cargo run -p clanker-cli --

# Or install globally (puts open-clanker in ~/.cargo/bin):
cargo install --path crates/cli
```

The `open-clanker` script is a bash wrapper that runs `cargo run --release -p clanker-cli`. It must be run from the project directory or you need the binary in PATH.

---

## 📋 Is Open Clanker Ready for Testing?

### ✅ Ready for Testing

| Area | Status | Notes |
|------|--------|------|
| **Unit tests** | ✅ 68 passing | core, config, gateway, agent, channels |
| **Gateway server** | ✅ Runs | HTTP, WebSocket, /health |
| **Agent integration** | ✅ In gateway | 4 providers: Anthropic, OpenAI, Grok, Groq |
| **Channel integration** | ✅ In gateway | Telegram `listen_with_tx` → agent → send |
| **CLI** | ✅ Functional | onboard, gateway, tui, config-validate |
| **Onboarding** | ✅ Wizard | API keys, Telegram, Discord |

### ⚠️ Partial / Not Yet

| Area | Status | Notes |
|------|--------|------|
| **Discord** | 🔄 Placeholder | Structure exists, needs full impl |
| **Send command** | ❌ Stub | "Not yet implemented" |
| **Storage (SQLite)** | ❌ Not started | Phase 4 |
| **Docker** | ❌ Not started | Phase 4 |
| **End-to-end tests** | ❌ None | Manual testing only |

### How to Test

```bash
# 1. Onboard (creates config.toml + .env)
./open-clanker onboard

# 2. Start gateway
source .env && ./open-clanker gateway

# 3. In another terminal: TUI client
./open-clanker tui

# 4. Or send a message via Telegram bot (if configured)
```

---

## 📖 Plans & Roadmap

### Phase 3A: Gateway Integration — IN PROGRESS

Per [GATEWAY_INTEGRATION_PLAN.md](./GATEWAY_INTEGRATION_PLAN.md):

- [x] Agent in gateway
- [x] Channels in gateway
- [x] Telegram `listen_with_tx` → mpsc → agent → channel.send
- [x] Processing loop
- [ ] WebSocket path: SendMessage → agent → response (partially wired)
- [ ] Discord full implementation

### Phase 4: Docker & Deployment

- [ ] Dockerfile
- [ ] docker-compose.yml
- [ ] Storage layer (SQLite)

### Phase 5: Testing & Validation

- [ ] Integration tests
- [ ] E2E: Telegram → AI response
- [ ] Performance benchmarks

### Phase 6: Documentation & Release

- [ ] API docs
- [ ] DEPLOY.md
- [ ] v1.0 release

---

## 🧪 Testing Summary

| Crate | Tests | Status |
|-------|-------|--------|
| clanker-core | 20 | ✅ |
| clanker-config | 11 | ✅ |
| clanker-gateway | 23 | ✅ |
| clanker-agent | 15 | ✅ |
| clanker-channels | 10 | ✅ |
| **Total** | **68** | ✅ |

Run: `cargo test --workspace`

---

## 🤖 Open Clanker vs OpenClaw

### Is Open Clanker a Lighter-Weight OpenClaw in Rust?

**Yes.** Per [ARCHITECTURE_COMPARISON.md](./ARCHITECTURE_COMPARISON.md) and [IMPLEMENTATION_CHECKLIST.md](./IMPLEMENTATION_CHECKLIST.md):

| Aspect | OpenClaw | Open Clanker |
|--------|----------|--------------|
| **Language** | TypeScript/Node.js | Rust |
| **Binary size** | ~500MB+ (Node + deps) | < 20MB |
| **Memory (idle)** | ~200–300MB | < 100MB |
| **Dependencies** | 780+ npm packages | ~50 Rust crates |
| **Channels** | 10+ (Telegram, Discord, Slack, etc.) | 2 (Telegram, Discord) |
| **AI providers** | Anthropic, OpenAI, etc. | Anthropic, OpenAI, Grok, Groq |

### Does It Have All OpenClaw Features?

**Core features: yes.** Extended features: simplified or deferred.

| Feature | OpenClaw | Open Clanker |
|---------|----------|--------------|
| Gateway (HTTP + WebSocket) | ✅ | ✅ |
| Multi-provider AI | ✅ | ✅ (4 providers) |
| Telegram | ✅ | ✅ |
| Discord | ✅ | 🔄 Partial |
| CLI | ✅ | ✅ |
| Config / onboarding | ✅ | ✅ |
| Slack, Signal, WhatsApp | ✅ | ⏳ Planned |
| Browser automation | ✅ | ❌ Deferred |
| Canvas rendering | ✅ | ❌ Deferred |
| Mobile apps | ✅ | ❌ (Linux-focused) |
| Voice | ✅ | ❌ Deferred |

Open Clanker focuses on the core AI assistant flow: channels → gateway → agent → response, with a smaller footprint and Linux-first deployment.

---

## 📁 Code Flow (How It Works)

```
User (Telegram/Discord)
    │
    ▼
Channel.listen_with_tx()  →  mpsc::Sender<Message>
    │
    ▼
Gateway processing loop: rx.recv()
    │
    ▼
processor::process_message(agent, &incoming)
    │  - core::Message → agent::AgentMessage
    │  - agent.chat() → AgentResponse
    │  - AgentResponse → core::Message
    ▼
channel.send(response)  →  User receives AI reply
```

### Key Files

- `crates/cli/src/main.rs` — CLI entry, commands
- `crates/gateway/src/server.rs` — Gateway, spawns channel listeners + processing loop
- `crates/gateway/src/processor.rs` — Message → agent → response
- `crates/agent/` — Anthropic, OpenAI, Grok, Groq clients
- `crates/channels/` — Telegram (teloxide), Discord (serenity)

---

## 🎯 What's Next

1. **Finish Phase 3A** — WebSocket SendMessage → agent → response
2. **Discord** — Full listen/send implementation
3. **Send command** — Implement actual message sending
4. **Storage** — SQLite for message persistence
5. **Docker** — Multi-stage Dockerfile, docker-compose
6. **Integration tests** — E2E Telegram → AI flow

---

## 🖼️ ASCII Art

The CLI and README now use:

- **Robot** — User-provided ASCII robot
- **Open_Clanker** — User-provided ASCII block font (88-style)

Run `./open-clanker` (no args) to see the welcome banner.
