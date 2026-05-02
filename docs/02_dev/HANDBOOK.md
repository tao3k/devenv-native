---
type: knowledge
title: "How to Query and Locate Ingested Papers"
category: "how-to"
tags:
  - how-to
  - knowledge
saliency_base: 6.0
decay_rate: 0.05
metadata:
  title: "How to Query and Locate Ingested Papers"
---

# How to Query and Locate Ingested Papers

This guide describes the end-to-end flow: user asks for papers (for example
"multimodal document parsing" or "ingest-related papers"), and the agent uses
knowledge tools or CLI flows to find and cite them.

---

## Flow Overview

```
User: "Find me papers about multimodal document parsing / ingest-related papers"
        │
        ▼
Agent calls knowledge tools or CLI helpers
        │
        ├── knowledge.recall(query="multimodal document parsing paper", limit=5)
        │   → Returns chunks with content, source, score from vector store
        │
        └── knowledge.search(query="RAG anything ing paper", mode="hybrid")
            → Returns merged LinkGraph + vector results; vector hits are from ingested docs
        │
        ▼
Agent interprets results and answers user
        → "The paper you ingested (e.g. arXiv 2510.12323) is in the knowledge base.
           Relevant snippets: [content]. The matching document can be identified
           from its source metadata or citation id."
```

---

## Knowledge Tools to Use

| Tool                 | When to use                                                                                  | Example                                                |
| -------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| **knowledge.recall** | Semantic search over the vector store (ingested PDFs, markdown, etc.)                        | Query: "multimodal document parsing overview abstract" |
| **knowledge.search** | Hybrid (LinkGraph + vector) or keyword-only; good when you want both notes and ingested docs | Query: "document parsing ingest paper", mode: "hybrid" |

Both can surface content from an ingested PDF. Recall returns `content`, `source`, `score`; search returns merged results with `source` and reasoning.

### Action-based recall

For long content, a single `knowledge.recall` with default `chunked=True` runs preview → fetch → all batches in one call, which can time out and cause memory accumulation. Use one step per tool call:

1. **start** – `knowledge.recall(query="...", chunked=True, action="start")` → preview only, returns `session_id` and `batch_count` (no full fetch; avoids memory spike).
2. **batch** – `knowledge.recall(session_id="<from start>", action="batch", batch_index=0)` … then `batch_index=1`, etc. → each call lazy-fetches and returns one batch (no full state in memory).
3. **full_document** – `knowledge.recall(chunked=True, action="full_document", source="2601.03192.pdf")` → returns **all chunks** for that document, sorted by `chunk_index`. Use when you need the **complete paper with no omission** (semantic search returns top-N and may miss chunks).

Each batch response is small; the LLM reads slice by slice. This avoids memory accumulation and token limits.

---

## How to "Locate" the Paper

- **After ingest**: The PDF is chunked and stored in the knowledge vector
  store with metadata such as `source` and `title`.
- **In recall results**: The `source` field in the API may be the chunk ID (e.g. UUID) depending on the vector backend. To show the user "which paper" a snippet came from, the agent can:
  1. Use the **content** of the recalled chunks (for example a paper title,
     figure caption, or abstract sentence) to infer the document.
  2. If the system exposes source metadata, use that to cite the document by
     source identifier, title, or arXiv id.

So "locating" the paper means: run recall/search with a natural-language query about the topic, then report the matching snippets and, when available, the document path or arxiv id from metadata or context.

---

## Example: User Asks for "Document Parsing / Ingest-Related Papers"

1. **User**: "帮我找寻文档解析或 ingest 相关的论文" (or: "Find me papers about
   document parsing or ingest-related work.")

2. **Agent** calls knowledge tools:
   - `knowledge.recall(query="document parsing or ingest pipeline paper", limit=5)`
   - Optionally: `knowledge.search(query="document parsing ingest paper", mode="hybrid")`

3. **Tool call returns** (example):
   - Chunks such as a figure caption, abstract sentence, or method summary
     with high score.
   - Other snippets about multimodal analysis, ingestion pipelines, and
     retrieval systems.

4. **Agent answers user**:
   - "The knowledge base contains a paper that matches your request. Relevant
     excerpts: [paste content]. The document can be cited from its source
     metadata or arXiv id."

---

## Prerequisites

- The target paper (or its PDF) must already be **ingested** via `knowledge.ingest_document` (e.g. after downloading the PDF to a local path). Download first, then ingest.
- Vector store must be available (e.g. after `omni sync knowledge` or `knowledge.ingest_document`); then recall/search work without extra runtime setup.

## Prefer Built-In Tooling

Prefer direct tool calls or CLI paths such as `omni skill run ...` and `omni knowledge recall ...`. Do not assume a separate external tool server is available.

## CLI: Fast path vs full skill

- **Fast path (recommended for CLI)**: `omni knowledge recall "query" [--limit N] [--json]`  
  Uses only the foundation vector store and embedding; **typically under 2s**. No kernel/skill stack.
- **Full skill**: `omni skill run knowledge.recall '{"query":"..."}'`  
  Loads full kernel and all skills (30–45s cold start); use when you need fusion boost (LinkGraph, KG).

## Timeouts (knowledge.recall)

- **Embedding**: Query embedding is limited by `knowledge.recall_embed_timeout_seconds` (default **18**). If the embedding service is slow or unreachable, recall falls back to a hash-based vector so the request returns within the limit (with potentially lower relevance) instead of hitting a client-side timeout.
- **Tool execution**: If recall still times out, use CLI `omni knowledge recall "your query" --limit 5` or increase `knowledge.recall_embed_timeout_seconds` (e.g. 25).

---

## Limit vs preview vs full read

- **limit** = how many **items** to return (for accuracy list or batch size). Not for "how much content" to read. Use **preview** to confirm recall is right; use a **workflow** to read long content in chunks.
- **preview** (`recall(..., preview=True, snippet_chars=150)`): returns only title, source, score, and first N chars per result → use to **verify accuracy** before pulling full content.
- **Long content in chunks** (papers, manuals, long docs): Recalled content is usually long, so `knowledge.recall` **default** is the chunked workflow (preview → fetch → batches). Chunking is consumed in memory: feed `batches[i]` to the LLM in turn so each slice stays in context. Response includes `preview_results`, `batches`, `all_chunks_count`, `results`. Use `chunked=False` for single-call search only.

## Research workflow: use ingested content

To **research or analyze** any long ingested content (paper, manual, long doc):

1. **Default (chunked)**: Call `knowledge.recall(query="…")` → get `preview_results`, `batches`, `results`; use preview to confirm accuracy, then feed `batches[i]` to the LLM one batch per turn.
2. **Single-call**: `knowledge.recall(query="…", chunked=False, limit=N)` for one batch of full chunks (no workflow).
3. **If recall times out**: Run `omni knowledge recall "…" --json` locally, or increase `knowledge.recall_embed_timeout_seconds`.

---

## Summary

- **Query**: Use natural-language queries with `knowledge.recall` or `knowledge.search` (hybrid).
- **Locate**: Use returned content + metadata (and, when available, document path) to tell the user which paper the snippets came from.
- **End-to-end**: Ingest PDF → User asks for papers → Agent uses knowledge tools → Agent returns snippets and paper identity (path/arxiv id).

---

title: "Run the Rust Agent (修仙道场)"
category: "how-to"
tags:

- how-to
- run
  saliency_base: 6.0
  decay_rate: 0.05

---

# Run the Rust Agent (修仙道场)

> Verification checklist for xiuxian-daochang: gateway, stdio, repl, memory, and session window.

**Quick start**: After `cargo build -p xiuxian-daochang`, use `omni agent --rust` or `omni gateway --rust` to run the Rust agent from the main CLI.

---

## E2E Validation Checklist

| Step                | Command / Action                                                                            | Status                                                 |
| ------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| 1. Build            | `cargo build -p xiuxian-daochang`                                                           | ✅                                                     |
| 2. Unit tests       | `cargo nextest run -p xiuxian-daochang`                                                     | ✅                                                     |
| 3. Gateway + LLM    | Start gateway; `curl POST /message`                                                         | Manual (needs `OPENAI_API_KEY` or `LITELLM_PROXY_URL`) |
| 4. Stdio            | `echo "msg" \| cargo run -p xiuxian-daochang -- stdio`                                      | Manual                                                 |
| 5. REPL             | `omni agent --rust` or `cargo run -p xiuxian-daochang -- repl`                              | Manual                                                 |
| 6. Integration test | `cargo nextest run -p xiuxian-daochang --test agent_integration --run-ignored ignored-only` | Manual (needs API key)                                 |

**Full E2E (Rust agent + LiteLLM)**: See §3 Gateway and §9 Integration test. Run `omni gateway --rust --webhook-port 8080`, then `curl` or run the integration test.

---

## Prerequisites

- **LLM**: `OPENAI_API_KEY` (or `ANTHROPIC_API_KEY` for Claude), or LiteLLM proxy (`LITELLM_PROXY_URL`, `XIUXIAN_DAOCHANG_MODEL`)
- **Valkey** (when `memory.persistence_backend=valkey`): configure `session.valkey_url` or set `VALKEY_URL`

---

## LLM Backend Selection

`xiuxian-daochang` supports two backend modes:

- `litellm_rs` (default): Rust `litellm-rs` provider path.
- `http`: direct OpenAI-compatible HTTP requests.

Selection priority:

1. `XIUXIAN_DAOCHANG_LLM_BACKEND` (env)
2. `agent.llm_backend` (settings)
3. default `litellm_rs`

Examples:

```bash
# force HTTP mode for a single run
XIUXIAN_DAOCHANG_LLM_BACKEND=http omni agent --rust

# use default Rust backend
XIUXIAN_DAOCHANG_LLM_BACKEND=litellm_rs omni agent --rust
```

Verify from logs:

- startup log: `"llm backend selected"` with `llm_backend` and `llm_backend_source`
- request log: `event="agent.llm.chat.dispatch"`
- completion log: `event="agent.llm.chat.completed"` (`elapsed_ms`, `tool_call_count`)
- failure log: `event="agent.llm.chat.failed"` (`elapsed_ms`, `error`)

### Performance Baseline (Rust LLM Path)

The Rust LLM client now applies:

- keep-alive HTTP pooling (`pool_max_idle_per_host=64`, idle timeout 90s)
- per-request connect timeout (5s)
- unified inference timeout from settings (`inference.timeout`)
- unified generation cap from settings (`inference.max_tokens`)
- optional in-flight concurrency gate (`inference.max_in_flight`)

This keeps latency tails bounded under high concurrency and prevents silent hangs.

### Rust-Native MiniMax (`litellm_rs`)

Use this path to keep inference fully in Rust (no Python `/v1/chat/completions` bridge):

```yaml
agent:
  llm_backend: "litellm_rs"
inference:
  provider: "minimax"
  api_key_env: "MINIMAX_API_KEY"
  base_url: "https://api.minimax.io/v1"
  model: "MiniMax-M2.5"
  max_in_flight: 32
```

Runtime override example:

```bash
VALKEY_URL=redis://127.0.0.1:6379/0 \
XIUXIAN_DAOCHANG_LLM_BACKEND=litellm_rs \
XIUXIAN_DAOCHANG_LLM_PROVIDER=minimax \
cargo run -p xiuxian-daochang -- repl --query "Only reply OK"
```

---

## 1. Build and unit tests

```bash
cargo build -p xiuxian-daochang
cargo nextest run -p xiuxian-daochang
```

Or run the full test pipeline (includes xiuxian-daochang):

```bash
just test
```

**Expected**: Build succeeds; all non-ignored tests pass (config, session, gateway validation, gateway HTTP 400/404, agent summarisation).

---

## 3. Gateway (HTTP)

**Terminal 1** — start gateway:

```bash
# Via omni CLI (after cargo build -p xiuxian-daochang)
omni gateway --rust --webhook-port 8080 --webhook-host 0.0.0.0

# Or directly
cargo run -p xiuxian-daochang -- gateway --bind 0.0.0.0:8080
```

**Terminal 2** — send a message:

```bash
curl -X POST http://127.0.0.1:8080/message \
  -H "Content-Type: application/json" \
  -d '{"session_id":"s1","message":"Say hello in one sentence."}'
```

**Expected**: JSON `{"output":"...","session_id":"s1"}` with model reply.

**Validation**: Empty `session_id` or `message` returns 400.

---

## 4. Stdio mode

```bash
echo "What is 2+2?" | cargo run -p xiuxian-daochang -- stdio --session-id test-session
```

**Expected**: One line of model output printed to stdout.

---

## 5. REPL (interactive or one-shot)

**Via omni CLI** (after `cargo build -p xiuxian-daochang`):

```bash
omni agent --rust
```

**One-shot** (direct):

```bash
cargo run -p xiuxian-daochang -- repl --query "List three programming languages."
```

**Interactive** (read-eval-print loop):

```bash
cargo run -p xiuxian-daochang -- repl
# Type a message, press Enter; repeat. Exit with Ctrl+C or EOF.
```

## 6. Scheduled Jobs (Recurring)

Run recurring background jobs directly from CLI:

```bash
cargo run -p xiuxian-daochang -- schedule \
  --prompt "research latest Rust actor runtime benchmarks" \
  --interval-secs 300 \
  --max-runs 3 \
  --schedule-id nightly-research \
  --session-prefix scheduler \
  --recipient scheduler \
  --wait-for-completion-secs 30
```

What this does:

- Submits one background job every `interval-secs`
- Stops after `max-runs` (or runs until Ctrl+C if omitted)
- Waits up to `wait-for-completion-secs` for in-flight jobs before exit

Use this for long-running recurring tasks without external cron orchestration.

---

## 7. Memory (recall + store)

When `config.memory` is set, the agent:

- Calls `two_phase_recall(user_message)` before the LLM and injects a system message with relevant past experiences
- Stores each turn as an episode (`try_store_turn`) and optionally consolidates when the window is full

**To enable**: Use an `AgentConfig` with `memory: Some(MemoryConfig::default())` (or custom path/embedding_dim). The main CLI uses memory by default when building the agent.

**Verification**: Run several turns in the same session; later turns should reflect earlier context (if recall finds relevant episodes).

---

## 8. Session window + consolidation

When `config.window_max_turns` and `config.consolidation_threshold_turns` are set:

- Session history is bounded (ring buffer)
- When turn count ≥ threshold, oldest `consolidation_take_turns` turns are drained
- Drained turns are stored in two forms:
  - one `xiuxian-memory-engine` episode (for recall)
  - one compact session summary segment (for prompt reuse in future turns)

### Compression settings (session)

```yaml
session:
  window_max_turns: 2048
  consolidation_take_turns: 32
  # consolidation_threshold_turns: 1536
  summary_max_segments: 8
  summary_max_chars: 480
  consolidation_async: true
  context_budget_tokens: 6000
  context_budget_reserve_tokens: 512
```

Environment overrides:

- `XIUXIAN_DAOCHANG_WINDOW_MAX_TURNS`
- `XIUXIAN_DAOCHANG_CONSOLIDATION_THRESHOLD_TURNS`
- `XIUXIAN_DAOCHANG_CONSOLIDATION_TAKE_TURNS`
- `XIUXIAN_DAOCHANG_SUMMARY_MAX_SEGMENTS`
- `XIUXIAN_DAOCHANG_SUMMARY_MAX_CHARS`
- `XIUXIAN_DAOCHANG_CONSOLIDATION_ASYNC`
- `XIUXIAN_DAOCHANG_CONTEXT_BUDGET_TOKENS`
- `XIUXIAN_DAOCHANG_CONTEXT_BUDGET_RESERVE_TOKENS`

`context_budget_tokens` + `context_budget_reserve_tokens` enable token-budget packing before each LLM call, so the latest turn is retained while older context is trimmed/truncated to stay within budget.

**Verification**: Long session (e.g. 50+ turns) with memory + window enabled; check that consolidation runs (e.g. via logs or memory store).

---

## 9. Graceful shutdown

**Gateway**: Press Ctrl+C (or send SIGTERM on Unix). Server stops accepting new connections and waits for in-flight requests to finish.

**Expected**: Log "gateway stopped"; no abrupt connection drops.

---

## 10. Integration test (real LLM + external tools)

Requires `OPENAI_API_KEY` and any optional external tool or embedding runtime your scenario depends on:

```bash
cargo nextest run -p xiuxian-daochang --test agent_integration --run-ignored ignored-only
```

---

## 11. Telegram Channel (Production-Ready)

`xiuxian-daochang channel` runs a high-concurrency Telegram bot with webhook mode, Valkey dedup, and user/group allowlists. **Suitable for commercial deployment.**

### Architecture

| Component     | Default             | Purpose                                                                                                           |
| ------------- | ------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Transport     | `webhook`           | Multi-instance, horizontal scaling                                                                                |
| Dedup backend | `valkey` (Redis)    | Idempotent webhook handling, no duplicate processing                                                              |
| Session key   | `chat_id` (default) | Default `chat` partition; configurable via `telegram.session_partition` (`chat_user`, `user`, `chat_thread_user`) |

### Configuration: `telegram.acl.allow`

ACL source is settings-only (`telegram.acl.*`). Channel CLI/env ACL overrides are removed.

#### `telegram.acl.allow.users` (private chats + who can talk in groups)

| Value                       | Meaning                                                       |
| --------------------------- | ------------------------------------------------------------- |
| `[]`                        | Deny all users                                                |
| `["*"]`                     | Allow all users (testing only)                                |
| `["123456789"]`             | Allow by **numeric Telegram user_id**                         |
| `["telegram:123456789"]`    | Allow by numeric user_id with `telegram:` prefix (normalized) |
| `["tg:123456789"]`          | Allow by numeric user_id with `tg:` prefix (normalized)       |
| `["123456789","987654321"]` | Multiple user IDs                                             |

#### `telegram.acl.allow.groups` (group chats — any member can talk if group allowed)

| Value                   | Meaning                                   |
| ----------------------- | ----------------------------------------- |
| `[]`                    | No groups allowed                         |
| `["*"]`                 | Allow all groups                          |
| `["-200123"]`           | Allow group by chat_id (negative = group) |
| `["-200123","-200456"]` | Multiple groups                           |

**How to get chat_id**: Add @userinfobot to the group, or check logs when an unauthorized message is rejected (logs show `chat_id=...`).

### settings.yaml examples

```yaml
telegram:
  acl:
    allow:
      users: ["123456789", "987654321"]
      groups: ["-200123456"]
    admin:
      users: ["123456789"]
    slash:
      global:
        users: ["123456789"]

  max_tool_rounds: 30
```

### .env example

```
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_WEBHOOK_SECRET=your_webhook_secret
```

### Polling (local testing)

```bash
TELEGRAM_BOT_TOKEN=<token> just agent-channel
```

`just agent-channel` now auto-starts local Valkey (`127.0.0.1:6379`) before
starting the polling runtime. Override with `just agent-channel <port>` or
`VALKEY_PORT`.

Or with explicit allowlist:

```bash
TELEGRAM_BOT_TOKEN=<token> \
just agent-channel
```

ACL allow/admin/slash rules are loaded from `telegram.acl.*` in settings files.

### Webhook (production)

**Requirement**: Public HTTPS URL. Use ngrok for dev, or a reverse proxy (nginx, Caddy) for production.

**One-shot** (ngrok + setWebhook + agent):

```bash
TELEGRAM_BOT_TOKEN=<token> just agent-channel-webhook
```

Settings are read from `packages/conf/settings.yaml` when env vars are not set.

**Production deployment** (manual):

1. Start Valkey: `just valkey-start`
   The local daemon keeps `pid`/`log` files under `.run/valkey` and writes `dump.rdb` under `.cache/valkey`.
   The managed launcher resolves those runtime paths against `$PRJ_ROOT` before handing them to Valkey, so pidfile and log writes stay inside the repository workspace even if the process manager changes its cwd.
   For OCI images, use `scripts/channel/valkey-launch.sh` as the startup entrypoint and `scripts/channel/valkey-healthcheck.sh` as the healthcheck; the checked-in example lives at `oci/valkey/Containerfile`, and the `just` wrappers remain host-side convenience layers.
2. Expose webhook (e.g. nginx → `http://127.0.0.1:18081`)
3. Set webhook: `curl "https://api.telegram.org/bot<TOKEN>/setWebhook?url=https://your-domain.com/telegram/webhook"`
4. Run agent:

   ```bash
   TELEGRAM_BOT_TOKEN=<token> \
   VALKEY_URL=redis://127.0.0.1:6379/0 \
   cargo run -p xiuxian-daochang -- channel \
     --mode webhook \
     --webhook-bind 0.0.0.0:18081 \
     --webhook-secret-token "<secret>"
   ```

### Background commands

- `/bg <prompt>` — Queue long-running task
- `/job <id>` — Job status
- `/jobs` — Queue health

### Valkey Stress Test

Run ignored stress tests that require a live Valkey backend:

```bash
just test-xiuxian-daochang-valkey-stress
```

Or directly:

```bash
VALKEY_URL=redis://127.0.0.1:6379/0 \
cargo nextest run -p xiuxian-daochang --test channels_webhook_stress --run-ignored ignored-only
```

Stop local Valkey when done:

```bash
just valkey-stop
```

Implementation note: `just` channel/valkey recipes are thin wrappers; operational logic lives in `scripts/channel/*.sh`.

### Debugging

When no logs or bot reply appear:

- Use `--verbose` or `-v` for detailed logs (shows user messages and bot replies).
- Check logs: `Webhook received Telegram update` → Telegram reached the server; `Parsed message, forwarding to agent` → message processed.
- If no logs: Telegram is not reaching the server. Ensure webhook URL is public and `setWebhook` was called.

### Telegram hardening tests

Run Telegram-specific robustness tests (Unicode-safe chunking, markdown fallback including API-level `ok=false`, caption MarkdownV2 fallback for single/media-group sends, transient send retries, topic routing via `chat_id:thread_id`, URL/local attachment marker routing, short-text caption routing with long-text fallback, `sendMediaGroup` batching/split/fallback behavior, polling error handling):

```bash
cargo nextest run -p xiuxian-daochang --test channels_telegram --test channels_telegram_chunking --test channels_telegram_markdown --test channels_telegram_media --test channels_telegram_polling
```

---

## 11. Scheduled Jobs (Recurring)

`xiuxian-daochang schedule` runs recurring prompts through the existing `JobManager` runtime.

One-shot finite run (useful for verification):

```bash
cargo run -p xiuxian-daochang -- schedule \
  --prompt "research compare rust actor runtimes" \
  --interval-secs 60 \
  --max-runs 3
```

Long-running scheduler (stop with Ctrl+C):

```bash
cargo run -p xiuxian-daochang -- schedule \
  --prompt "collect system summary" \
  --interval-secs 300
```

Notes:

- Scheduler submissions reuse background job workers (`JobManager`).
- `--max-runs` controls submission count; without it, scheduler runs until interrupted.
- `--wait-for-completion-secs` controls post-stop drain time for in-flight jobs.

---

## Env vars

| Var                      | Purpose                                                                                   |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| `OPENAI_API_KEY`         | API key for OpenAI-compatible endpoint                                                    |
| `ANTHROPIC_API_KEY`      | For Claude endpoints                                                                      |
| `LITELLM_PROXY_URL`      | **Recommended.** Chat completions URL (e.g. `http://127.0.0.1:4000/v1/chat/completions`). |
| `XIUXIAN_DAOCHANG_MODEL` | Model id (e.g. `gpt-4o-mini`)                                                             |
| `TELEGRAM_BOT_TOKEN`     | Telegram bot token for `xiuxian-daochang channel`                                         |
| `VALKEY_URL`             | Fallback Valkey/Redis URL when `session.valkey_url` is not configured                     |

**First-time setup**: Set `LITELLM_PROXY_URL` (or run LiteLLM and point to it) and `OPENAI_API_KEY` before starting the agent.

---

## CLI reference

```bash
xiuxian-daochang gateway --help
xiuxian-daochang stdio --help
xiuxian-daochang repl --help
xiuxian-daochang channel --help
xiuxian-daochang schedule --help
```

**Gateway options**: `--bind`, `--turn-timeout`, `--max-concurrent`

## **Channel options**: `--bot-token`, `--mode` (polling|webhook), `--webhook-bind`, `--webhook-path`, `--webhook-secret-token`, `--webhook-dedup-backend` (memory|valkey), `--valkey-url`, `--webhook-dedup-ttl-secs`, `--webhook-dedup-key-prefix`, `-v`/`--verbose` (show user messages and bot replies in logs)

title: "Verify MemRL Memory (Self-Evolution) via REPL or Telegram"
category: "how-to"
tags:

- how-to
- verify
  saliency_base: 6.0
  decay_rate: 0.05

---

# Verify MemRL Memory (Self-Evolution) via REPL or Telegram

> **Purpose**: Validate that xiuxian-daochang's memory (MemRL-inspired: two-phase recall, Q-learning, store_episode) works. The "self-evolution" effect: high-utility episodes surface in recall; low-utility ones are deprioritized.

---

## Prerequisites

- LLM configured (`OPENAI_API_KEY` or `LITELLM_PROXY_URL`)
- external tool setup is no longer required for the default runtime path

---

## Recommended: REPL (Two Commands)

Memory persists to disk (`memory/`). Run two one-shot commands with the same `--session-id`:

```bash
# Turn 1: Store episode
cargo run -p xiuxian-daochang -- repl --query "Remember: my favorite number is 42." --session-id mem-test

# Turn 2: Recall (new process loads same memory store)
cargo run -p xiuxian-daochang -- repl --query "What's my favorite number?" --session-id mem-test
```

**Expected**: Turn 2 reply includes "42" (recalled from Turn 1).

**Embedding**: When an embedding HTTP server is running on the configured endpoint, xiuxian-daochang uses it for semantic encoding. Otherwise it falls back to hash-based encoder. Set `OMNI_EMBEDDING_URL` to override the default `http://127.0.0.1:18501`.

### macOS Apple Silicon: `mistralrs` Metal build notes

- In this repository, `xiuxian-llm` backend is selected by target OS at compile time:
  - macOS: `mistralrs` with `metal`
  - Linux: `mistralrs` with `cuda`
  - Other OS: `mistralrs` default backend (CPU path)
- For macOS Metal toolchain prerequisites, follow the Modular guide:  
  <https://puzzles.modular.com/howto.html#macos-apple-silicon>
- Quick verification commands:

```bash
xcodebuild -version
xcrun -sdk macosx metal
```

- If `xcrun ... metal` reports missing Metal toolchain, install it:

```bash
xcodebuild -downloadComponent MetalToolchain
```

- `MISTRALRS_METAL_PRECOMPILE=0` means "skip build-time Metal kernel precompile".
  It is a build-stability/build-speed knob, not a switch that disables Metal runtime execution.
- For fastest cold start on a fully provisioned local macOS machine, keep precompile enabled
  (unset `MISTRALRS_METAL_PRECOMPILE`).
- For CI/Nix environments where `metal` tool is unavailable, set `MISTRALRS_METAL_PRECOMPILE=0`
  to avoid build failures and allow runtime compilation fallback.

---

## Alternative: Telegram

- `omni channel --rust` (or `cargo run -p xiuxian-daochang -- channel`) running with bot token configured
- Same Telegram chat = same session (`telegram:{chat_id}`)
- For local testing, use `--mode polling`; for production with a public URL, use `--mode webhook` (see [Run the Rust Agent §10](../how-to/run-rust-agent.md#10-telegram-channel))

### Automated Telegram validation suite

Use the Pythonized suite for repeatable black-box validation:

```bash
# Quick command-path checks
python3 scripts/channel/test_xiuxian_daochang_memory_suite.py --suite quick --max-wait 90 --max-idle-secs 40 --username tao3k

# Full live suite: includes memory self-evolution DAG validation by default
python3 scripts/channel/test_xiuxian_daochang_memory_suite.py --suite full --max-wait 90 --max-idle-secs 40 --username tao3k

# Full suite but skip DAG stage (command probes + Rust regressions only)
python3 scripts/channel/test_xiuxian_daochang_memory_suite.py --suite full --skip-evolution
```

### CI gate runner (mock Telegram + local webhook runtime)

Use the orchestrator for repeatable CI/local gate execution:

```bash
# PR-level quick gate (command-path + Rust regressions, no DAG)
python3 scripts/channel/test_xiuxian_daochang_memory_ci_gate.py --profile quick

# Nightly gate (full suite + DAG quality + session matrix + benchmark)
python3 scripts/channel/test_xiuxian_daochang_memory_ci_gate.py --profile nightly --max-memory-stream-read-failed-events 0

# Debug matrix/benchmark path without DAG stage
python3 scripts/channel/test_xiuxian_daochang_memory_ci_gate.py --profile nightly --skip-evolution --skip-benchmark
```

The gate runner automatically starts/stops Valkey, a local Telegram API mock server, and the local webhook runtime.
By default it uses an auto-generated Valkey key prefix per run (`XIUXIAN_DAOCHANG_SESSION_VALKEY_PREFIX`)
to isolate CI traffic from any other local runtime sharing the same `VALKEY_URL`.
By default it also writes run-scoped log/report files (profile + run suffix), so concurrent quick/nightly runs do not overwrite each other.
If Valkey is already running before the gate starts, the gate will not shut it down in cleanup.
Use `--valkey-prefix <prefix>` to override the default isolation prefix when needed.
Use explicit `--runtime-log-file`, `--mock-log-file`, and `--*-report-*` options only when you intentionally need fixed output paths.

Benchmark note: the nightly benchmark issues `/reset` and `/session feedback ...` control commands.
The benchmark `--user-id` must map to an admin-capable Telegram identity in runtime policy, otherwise the run fails with `admin_required`.

---

## Scenario 1: Memory Recall (Multi-Turn)

**Validates**: Episodes are stored; two_phase_recall injects them into context.

| Turn | You send                              | Expected                               |
| ---- | ------------------------------------- | -------------------------------------- |
| 1    | "Remember: my favorite number is 42." | Agent acknowledges                     |
| 2    | "What's my favorite number?"          | Agent says "42" (recalled from Turn 1) |

**Why it works**: Turn 1 is stored as episode (intent + experience + outcome=completed, Q=1.0). Turn 2 triggers `two_phase_recall("What's my favorite number?")` → semantic match finds Turn 1 → injected as system context → LLM sees it.

---

## Scenario 2: Self-Purification (Q-Value Filtering)

**Validates**: Two-phase recall prefers high-Q episodes. When similar intents have different outcomes, successful ones rank higher.

**Mechanism**:

- Each turn: `store_episode` + `update_q(reward)`. Success → Q↑; response containing "error"/"failed" → Q↓.
- Recall: Phase 1 semantic → Phase 2 rerank by `(1-λ)×similarity + λ×Q`. High-Q episodes surface.

**How to observe** (requires multiple similar intents with mixed outcomes):

1. **Turn 1**: "How do I fix a connection timeout?" → Agent gives a good answer → stored, Q=1.0
2. **Turn 2**: Ask something that leads to an error response (e.g. trigger a tool failure) → stored, Q=0.0
3. **Turn 3**: "How do I fix a connection timeout?" again → Recall should prefer Turn 1 (high Q) over Turn 2 (low Q); response should reflect the successful pattern

**Note**: Outcome is inferred from assistant message (contains "error"/"failed"/"exception" → failure). Tool failures often produce such text, so they get Q=0.0.

---

## Scenario 3: Consolidation (Long Session)

**Validates**: When session window is full, oldest turns are consolidated into one episode.

**Requirement**: Agent must be built with `window_max_turns` and `consolidation_threshold_turns` set. Currently the default config has these as `None`, so consolidation does not run. To enable:

- Modify `build_agent` in `main.rs` (or add config): `window_max_turns: Some(20)`, `consolidation_threshold_turns: Some(10)`, `consolidation_take_turns: 5`
- Then: 10+ turns in same session → consolidation drains oldest 5 → summarises → stores as one episode with reward

---

## Quick Test (Scenario 1)

**REPL** (with embedding server for semantic recall):

```bash
# With embedding server on 18501: semantic recall works
cargo run -p xiuxian-daochang -- repl --query "Remember: my favorite number is 42." --session-id mem-test
cargo run -p xiuxian-daochang -- repl --query "What's my favorite number?" --session-id mem-test

# Without embedding server: recall/store is skipped (no hash fallback), so expect no memory hit
cargo run -p xiuxian-daochang -- repl --query "What is my favorite number? (Answer: 42)" --session-id mem-test
cargo run -p xiuxian-daochang -- repl --query "What's my favorite number?" --session-id mem-test
```

**Expected**: Second reply includes "42".

**Telegram**: Start `omni channel --rust`, send the same two messages in sequence.

---

## Troubleshooting

| Issue                  | Cause                               | Fix                                                                                                             |
| ---------------------- | ----------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Agent doesn't recall   | Memory disabled or store path wrong | Check `config.memory` in agent; default path `PRJ_CACHE_HOME/xiuxian-memory-engine/` (see dirs.py / xiuxian-io) |
| No episodes stored     | store_episode fails                 | Check disk space; ensure embedding_dim matches                                                                  |
| Recall returns nothing | No similar past episodes            | Run Scenario 1 first to create episodes                                                                         |

---

## References

- [Omni-Memory](../reference/xiuxian-memory-engine.md) — implementation
- [MemRL vs Omni-Memory](../workflows/research-memrl-vs-xiuxian-memory-engine.md) — research comparison
- [Unified Execution Engine](../reference/unified-execution-engine-design.md) — MemRL integration
