# Gateway Integration Plan: Agent + Channels

## Overview

The gateway currently runs as a WebSocket/HTTP server but **does not** wire up the agent or channels. Messages are not processed by AI, and Telegram/Discord bots are not started. This document describes what needs to be done and the implementation plan.

---

## Current State

| Component | Status | Gap |
|-----------|--------|-----|
| **Gateway** | HTTP, WebSocket, /health work | No agent, no channels |
| **Agent crate** | 4 providers (Anthropic, OpenAI, Grok, Groq) | Not used by gateway |
| **Channels crate** | Telegram (teloxide), Discord (serenity) | Telegram echoes only; no gateway integration |
| **Message flow** | WebSocket SendMessage increments counter | No agent call, no channel send |

---

## Target Flow

```
User (Telegram)
    │
    ▼
Telegram Bot (receives message)
    │
    ▼
Gateway (receives core::Message via mpsc)
    │
    ▼
Agent.chat() → AI response
    │
    ▼
Channel.send(response) → User receives reply
```

---

## What Needs to Be Done

### 1. Gateway Holds Agent + Channels

- Add `clanker-agent` and `clanker-channels` as gateway dependencies
- Create agent from config via `AgentFactory::create_from_config(config.agent)`
- Create channel instances from config (Telegram, Discord based on config)
- Store in `AppState`: `agent: Box<dyn Agent>`, `channels: Vec<Box<dyn Channel>>`

### 2. Incoming Message Pipeline

- **Channels** must forward incoming messages to the gateway (not echo)
- Add `listen_with_tx(tx: mpsc::Sender<Message>)` to Channel trait (or equivalent)
- **Telegram**: Modify `listen()` to convert teloxide message → core::Message, send to tx, do NOT echo
- **Gateway**: Spawn channel listeners in background tasks, each sending to a shared `mpsc::Receiver`
- **Gateway**: Main processing loop: receive Message → call agent.chat() → build response Message → channel.send()

### 3. Message Conversion

- **core::Message → agent::AgentMessage**: `Message { text, sender, channel_id, ... }` → `AgentMessage { role: User, content: text }`
- **agent::AgentResponse → core::Message**: `AgentResponse { content }` → `Message { channel_type, channel_id, sender: "assistant", text: content }`

### 4. Channel Selection

- When sending response, find the channel that matches `message.channel_type` (Telegram or Discord)
- Use that channel's `send()` to deliver the AI response

### 5. Discord Status

- Discord `listen()` and `send()` are placeholders
- Phase 1: **Telegram only** for end-to-end
- Discord can remain placeholder until Phase 2

---

## Implementation Plan (Efficient Order)

### Phase A: Gateway + Agent (WebSocket path first)

1. Add `clanker-agent` to gateway Cargo.toml
2. Create agent in `AppState` from config
3. Add message processor module: `process_message(msg: Message) -> Message`
4. Wire WebSocket `SendMessage` handler: on receive → convert to core::Message → call agent → send response back via WebSocket
5. **Test**: WebSocket client sends message, receives AI response (no channels yet)

### Phase B: Channel Integration (Telegram)

1. Add `clanker-channels` to gateway Cargo.toml
2. Add `listen_with_tx` (or callback) to Channel trait + Telegram implementation
3. Telegram: replace echo with `tx.send(core_msg)` in repl handler
4. Gateway: create channels from config, spawn `listen_with_tx` tasks with shared mpsc
5. Gateway: add processing loop that receives from mpsc → agent → channel.send()
6. **Test**: Send message on Telegram → receive AI response

### Phase C: Polish + Discord (optional)

1. Wire WebSocket and channel paths through same processor
2. Add Discord listen/send implementation (or defer)
3. Error handling, logging, shutdown propagation

---

## File Changes Summary

| File | Changes |
|------|---------|
| `crates/gateway/Cargo.toml` | Add clanker-agent, clanker-channels, tokio mpsc |
| `crates/gateway/src/state.rs` | Add agent, channels, incoming_tx to AppState |
| `crates/gateway/src/server.rs` | Create agent/channels, spawn processing loop |
| `crates/gateway/src/processor.rs` | **NEW** – message processing (agent call, conversion) |
| `crates/gateway/src/handlers.rs` | Wire SendMessage to processor |
| `crates/channels/src/lib.rs` | Add `listen_with_tx` or equivalent to Channel trait |
| `crates/channels/src/telegram.rs` | Implement listen_with_tx, forward to tx instead of echo |

---

## Dependencies / Compatibility

- **Agent**: Uses `AgentConfig` from clanker-config; `AgentFactory::create_from_config` takes that
- **Channels**: `ChannelFactory::create` needs `(ChannelType, token)`; config has `channels.telegram.bot_token`, etc.
- **Core Message**: `Message::new(channel_type, channel_id, sender, text)` for responses

---

## Success Criteria

- [ ] `open-clanker onboard` → `source .env && open-clanker gateway`
- [ ] Send message on Telegram bot → receive AI (Claude/OpenAI/etc.) response
- [ ] WebSocket client can send message and receive AI response
- [ ] All existing tests pass
