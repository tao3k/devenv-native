---
type: knowledge
title: "Session Interactive Policy & Human-Centric Design"
category: "standards"
tags:
  - interaction
  - ux
  - session-management
  - openclaw-inspired
metadata:
  title: "Session Interactive Policy & Human-Centric Design"
---

# Session Interactive Policy & Human-Centric Design

This document defines the behavioral standards for `xiuxian-daochang` interaction, heavily inspired by the human-centric optimizations found in the `OpenClaw` research.

## 1. Interaction Modes (`QueueMode`)

| Mode            | Behavior                                                 | Use Case                                            |
| :-------------- | :------------------------------------------------------- | :-------------------------------------------------- |
| **`Queue`**     | Messages are processed one by one.                       | Standard task execution.                            |
| **`Interrupt`** | **[High Priority]** New message kills the previous task. | Correcting a model that has gone off-rails.         |
| **`Steer`**     | Input is injected as a "Hint" into the ongoing process.  | Long-running planning tasks.                        |
| **`Collect`**   | Wait for $X$ ms of silence before replying.              | Group chats where users send multiple short bursts. |

## 2. Session Reset Logic

### 2.1 Implicit Reset (The "Stale" State)

When a message arrives, the system checks `updated_at`:

```text
If (now - updated_at) > config.idle_timeout_mins:
    Archive old context -> Initialize NEW context
    Insert system notice: "Previous session expired due to inactivity."
```

### 2.2 Scheduled Reset (Daily Cleanse)

Configurable via `daily_reset_hour`. Ensures the agent starts every morning with a "Tabula Rasa" (Clean Slate) for maximum reasoning accuracy.

## 3. Human-in-the-Loop (HITL) Workflow

For tools marked as `restricted: true` or when `send_policy: "deny"` is active:

1. Agent generates a "Proposed Action" (e.g., `rm -rf ./tmp`).
2. The runtime policy layer intercepts and creates a **Pending Approval** record in Valkey.
3. Telegram/Discord sends a message with **[Approve]** and **[Deny]** buttons.
4. Logic remains suspended until a callback is received.

## 4. Behavioral Tuning Knobs

Users can adjust these parameters per-session to control the "Character" of the interaction:

- **`ThinkingLevel`**: Maps to `max_tokens` and CoT (Chain of Thought) prompt depth.
- **`VerboseLevel`**: Adjusts system instructions to prefer brevity or detailed explanations.
- **`ElevationLevel`**: Controls the threshold for tool execution permissions.

## 5. Metadata-Driven UX (Auto-Titles)

To aid in session recovery after restart:

- **First-Msg-Title**: The first 60 characters of the opening message are extracted as the session's `displayName`.
- **Iconography**: `group_activation` status ("Mention-only" vs "Always-listening") should be reflected in the session metadata for UI display.

## 6. Runtime Progress Snapshot (2026-02-28)

The policy is now partially operational in live channels with concrete command-path behavior:

- **`/session scope` alias is active** and maps to `/session partition` for both Telegram and Discord command handlers.
- **Authorization is stable** because ACL selectors are evaluated against canonical command identity.
- **Reply text is normalized** to `channel` scope language in both channels.
- **Partition persistence is available** via user TOML when explicitly enabled.
- **Discord mention policy is available** with default + per-channel TOML control and runtime
  `/session mention [status|on|off|inherit] [json]` updates for the current recipient channel.

### 6.1 Persistence Controls

User TOML:

- `telegram.session_partition_persist`
- `discord.session_partition_persist`
- `discord.require_mention`
- `discord.require_mention_persist`
- `discord.channels."<recipient>".require_mention`

Environment overrides:

- `XIUXIAN_DAOCHANG_TELEGRAM_SESSION_PARTITION_PERSIST`
- `XIUXIAN_DAOCHANG_DISCORD_SESSION_PARTITION_PERSIST`
- `XIUXIAN_DAOCHANG_DISCORD_REQUIRE_MENTION`
- `XIUXIAN_DAOCHANG_DISCORD_REQUIRE_MENTION_PERSIST`

### 6.2 Live Operator Check

Recommended quick check sequence:

1. Enable `telegram.session_partition_persist = true` in user `xiuxian.toml`.
2. Send `/session scope user json`.
3. Validate persisted value in `.config/xiuxian-artisan-workshop/xiuxian.toml`.
4. Send `/session scope json` to confirm runtime and persisted mode match.

### 6.3 Discord Mention Gate Check

Recommended quick check sequence:

1. Enable `discord.require_mention = true` in user `xiuxian.toml`.
2. In one Discord channel, send `/session mention off json` to clear mention gating for that
   recipient only.
3. In another Discord channel, leave inherited policy active and verify plain guild text is ignored
   unless the bot is directly mentioned or replied to.
4. Validate persisted channel-specific override under
   `discord.channels."<recipient>".require_mention` when
   `discord.require_mention_persist = true`.
