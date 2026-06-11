---
type: knowledge
title: "ADR-004: Advanced Session Governance & Lifecycle"
status: "Accepted"
date: "2026-02-26"
category: "architecture"
tags:
  - zhenfa
  - session
  - privacy
  - lifecycle
metadata:
  title: "ADR-004: Advanced Session Governance & Lifecycle"
---

# ADR-004: Advanced Session Governance & Lifecycle

## 1. Context and Problem Statement

As `xiuxian-daochang` moves into multi-user environments (Telegram/Discord groups), our current "one-size-fits-all" session model is insufficient:

- **Privacy Risk**: In a group, user A's private data might be recalled into user B's context.
- **Cognitive Decay**: Infinite context windows eventually lead to model confusion and high token costs.
- **Lack of Control**: Users cannot stop a rambling model or enforce an approval step for sensitive tools.

Inspired by the `OpenClaw` architecture, we need a robust governance model for session lifecycle and interaction policies.

## 2. Decision

We will implement a tiered **Session Governance Framework** within `xiuxian-zhenfa` and `xiuxian-daochang`.

### 2.1 Session Scoping (Isolation)

- **`Per-Sender` (Default)**: Unique context per user, even in shared groups.
- **`Global`**: Shared whiteboard for collaborative tasks.
- **`Thread/Topic`**: Isolation based on platform-native thread IDs.

### 2.2 Freshness & Reset Policy

- **Idle Timeout**: Sessions are marked "Stale" after $N$ minutes of inactivity (default 60).
- **Daily Reset**: Automated context clearing at a specific hour (e.g., 04:00 AM) to maintain model sharpness.
- **Manual Reset**: Standardized triggers like `/new`, `/reset`.

### 2.3 Interactive Control (Queue Modes)

- **`Interrupt`**: New input from the same user immediately sends an `abort` signal to the current LLM inference.
- **`Human-in-the-loop (HITL)`**: A "Gatekeeper" state where tools or messages require explicit user approval via UI buttons before proceeding.

## 3. Consequences

### Positive

- **Guaranteed Privacy**: Prevents cross-user data leakage in group settings.
- **Cost Efficiency**: Automated resets prevent context bloat and token waste.
- **Safety**: HITL mode allows safe usage of destructive tools (system commands, file deletions).

### Negative

- **Complexity**: Requires a stateful `InterruptController` and more complex session metadata in Valkey.

## 4. Implementation Status (2026-02-28)

The following ADR decisions are now implemented in `xiuxian-daochang` and verified with both automated and live channel tests:

- **Session command aliasing**: `/session scope` is accepted as an alias of `/session partition`.
- **ACL canonicalization**: authorization checks evaluate the canonical selector (`/session partition`) so alias usage remains policy-safe.
- **Channel-scoped status wording**: session partition replies now use the term `channel` scope (instead of `runtime`) in Telegram and Discord outputs.
- **Optional persistence to user TOML**: partition changes can be persisted to user config when enabled.

### 4.1 Config Knobs

User config (`.config/xiuxian-artisan-workshop/xiuxian.toml`):

- `telegram.session_partition_persist = true|false`
- `discord.session_partition_persist = true|false`

Environment overrides:

- `XIUXIAN_DAOCHANG_TELEGRAM_SESSION_PARTITION_PERSIST`
- `XIUXIAN_DAOCHANG_DISCORD_SESSION_PARTITION_PERSIST`

Default behavior remains non-persistent unless explicitly enabled.

### 4.2 Test Evidence

Validated with `cargo nextest`:

- `direnv exec . cargo nextest run -p xiuxian-daochang --test config_settings`
- `direnv exec . cargo nextest run -p xiuxian-daochang --test channels_managed_commands`
- `direnv exec . cargo nextest run -p xiuxian-daochang --test test_support_parsers`
- `direnv exec . cargo nextest run -p xiuxian-daochang --lib runtime_handle_inbound_session_scope_alias_updates_mode`
- `direnv exec . cargo nextest run -p xiuxian-daochang --lib process_discord_message_handles_scope_alias_and_updates_mode`
- `direnv exec . cargo nextest run -p xiuxian-daochang --lib runtime_handle_inbound_session_partition_status_and_toggle`
- `direnv exec . cargo clippy -p xiuxian-daochang -- -W clippy::too_many_lines`

### 4.3 Live Webhook Validation

Live Telegram webhook validation confirmed persistence and read-back:

1. Send `/session scope user json`.
2. Confirm `telegram.session_partition = "user"` written to `.config/xiuxian-artisan-workshop/xiuxian.toml`.
3. Send `/session scope json` and verify response `current_mode = "user"`.
4. Restore baseline with `/session scope chat_user json`.
