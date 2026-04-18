# xiuxian-daochang

Minimal Rust agent loop (Phase B): one user turn with LLM and optional external tool integrations.

## Features

- **Config** (`AgentConfig`): inference API URL, model, API key (env or field), external-tool server list (`tool_servers`), `max_tool_rounds`, optional `window_max_turns`.
- **Session**: in-memory `SessionStore` per `session_id`; or when `window_max_turns` is set, **omni-window** (ring buffer) for bounded history and scalable context (1k–10k turns).
- **LLM** (`LlmClient`): OpenAI-compatible chat completions with optional tool definitions and `tool_calls` parsing.
- **Agent** (`Agent`): `run_turn(session_id, user_message)` — builds messages, optionally fetches external tools, calls LLM, handles tool calls, repeats until no tool_calls or `max_tool_rounds`.
- **Bounded native search** (`wendao.search`, alias `knowledge.search`): a Daochang-owned native tool that invokes the bundled Qianji Wendao SQL authoring workflow against a configured gateway endpoint.

## Usage

```rust
use xiuxian_daochang::{Agent, AgentConfig, ToolServerEntry};

let config = AgentConfig {
    inference_url: "https://api.openai.com/v1/chat/completions".to_string(),
    model: "gpt-4o-mini".to_string(),
    api_key: None, // uses OPENAI_API_KEY from env
    tool_servers: vec![ToolServerEntry {
        name: "local".to_string(),
        url: Some("http://127.0.0.1:3002/sse".to_string()),
        command: None,
        args: None,
    }],
    max_tool_rounds: 10,
};

let agent = Agent::from_config(config).await?;
let reply = agent.run_turn("my-session", "What's the weather?").await?;
```

## Bounded Wendao Search

`wendao.search` and its knowledge-facing alias `knowledge.search` are mounted
when Daochang boots. At call time, the tool reads `[wendao_gateway]` from
`xiuxian.toml`, resolves the effective project root from explicit tool
arguments, session-id overrides, or a configured default, then passes that
scope into the bundled Qianji workflow.

```toml
[wendao_gateway]
query_endpoint = "http://127.0.0.1:18093/query"
default_project_root = "/path/to/project"

[wendao_gateway.session_project_roots]
"discord:search-thread" = "/path/to/project"
```

The native tool calls the bounded workflow preset that lives in
`xiuxian-qianji`; Daochang owns configuration, session scope resolution, and
result formatting. The preferred tool argument is `query`; the legacy `request`
field remains accepted for compatibility. The legacy zhenfa bridge
intentionally does not register `wendao.search`; direct gateway native-tool
dispatch is the only supported ownership path. Native-only agents advertise
both `knowledge.search` and `wendao.search` to the LLM in deterministic name
order, with `knowledge.search` as the preferred knowledge-facing entrypoint,
even when no external tool runtime is configured, so real tool-calling does not depend on
MCP/external-tool startup.

## Discord Mention Policy

Discord can require explicit bot triggers for guild text while leaving selected
channels open.

```toml
[discord]
require_mention = true
require_mention_persist = true

[discord.channels."123456789012345678"]
require_mention = false
```

Runtime control uses the current recipient channel:

- `/session mention`
- `/session mention on`
- `/session mention off`
- `/session mention inherit`

Slash interactions and slash-style managed commands remain usable even when
mention gating is enabled.

Foreground Discord turns that exceed the runtime timeout are automatically
requeued as background jobs. Daochang replies with a short background-job
handoff immediately and posts the completion back into the same channel when
the background run finishes.

## Reusing LiteLLM (no extra bridge)

The agent is an **OpenAI-compatible HTTP client**. To reuse [LiteLLM](https://docs.litellm.ai/) (100+ providers: OpenAI, Anthropic, Ollama, etc.), point `inference_url` at the LiteLLM proxy. No separate bridge process or SDK:

1. Start LiteLLM: `litellm --port 4000` (or set `LITELLM_PROXY_URL`).
2. Set `inference_url` to `http://127.0.0.1:4000/v1/chat/completions` (or use `AgentConfig::litellm("gpt-4o-mini")` which reads `LITELLM_PROXY_URL` and `XIUXIAN_DAOCHANG_MODEL`).
3. Use any model string LiteLLM supports: `gpt-4o`, `claude-3-5-sonnet`, `ollama/llama2`, etc. API keys are usually set in LiteLLM’s environment.

```rust
// Prefer LiteLLM so one endpoint can route to OpenAI, Anthropic, Ollama, etc.
let config = AgentConfig::litellm("gpt-4o-mini");
let agent = Agent::from_config(config).await?;
```

## Example

```bash
export OPENAI_API_KEY=sk-...
# Optional: use LiteLLM proxy (then set LITELLM_PROXY_URL or use default :4000)

cargo run -p xiuxian-daochang --example one_turn -- "Say hello in one sentence."
```

## Tests

- Unit: `cargo test -p xiuxian-daochang --test config_and_session`
- Integration (runtime-default live LLM + optional external tools): `XIUXIAN_DAOCHANG_LIVE_AGENT_INTEGRATION=1 direnv exec . cargo test -p xiuxian-daochang unit::agent_integration::test_agent_one_turn_with_llm_and_tools -- --exact --nocapture`
- Focused live native-tool proof (runtime-default provider): `XIUXIAN_DAOCHANG_LIVE_LLM=1 direnv exec . cargo test -p xiuxian-daochang unit::agent_suite::agent::native_tools_wendao_search_live::runtime_default_llm_performs_native_wendao_search_tool_call -- --exact --nocapture`
- Valkey-backed live tests (env-gated): `XIUXIAN_DAOCHANG_LIVE_VALKEY=1 VALKEY_URL=redis://127.0.0.1:6379/0 direnv exec . cargo test -p xiuxian-daochang unit::telegram_session_gate::distributed_same_session_is_serialized_across_gate_instances -- --exact --nocapture`

## Plan

See [docs/how-to/run-rust-agent.md](../../../docs/how-to/run-rust-agent.md) for the verification checklist. Phase B + C done (agent loop + gateway).
