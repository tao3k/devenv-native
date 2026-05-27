# Core Boundary

:PROPERTIES:
:ID: f92521584a38a3602c16ad2eeabbd32eb4f60eb9
:TYPE: CORE
:STATUS: DRAFT
:END:

Architecture boundary note for the `xiuxian-llm` library crate. Capture core responsibilities, integration edges, and invariants here.

The active project-policy gate uses `rust-lang-project-harness` without disabled rules. The public module tree exposes explicit embedding, LLM, model-runtime, and web boundaries; provider-specific Anthropic message handling is split into focused child modules for routing, media normalization, HTTP retry, request conversion, response parsing, and provider construction.

## Wendao Model Routing Contract

`xiuxian-llm` owns the shared Wendao model routing contract used by Gateway,
transport adapters, and analyzer execution adapters. The default feature
profile enables both `model-routing` and `provider-litellm`; narrow consumers
may still compile only the route contract with `--no-default-features
--features model-routing`.

The route contract defines `WendaoRouteIntent`, `WendaoModelDecision`, routing
mode parsing, vLLM-SR endpoint configuration keys, and stable Arrow Flight
metadata keys. Gateway remains responsible for task semantics, source evidence,
precision admission, and route-decision acquisition. Analyzer adapters consume
the selected backend metadata and execute the requested backend; they do not
own model/provider selection policy.

`xiuxian-llm` ships source-owned system defaults under `src/resource/`. The
LLM runtime default backend is `litellm`; the default chat provider is direct
DeepSeek at `https://api.deepseek.com/v1`, while OpenRouter and local
OpenAI-compatible providers remain named alternatives. Root `wendao.toml` may
overlay `[model_routing]` to choose project-specific route mode, vLLM-SR base
URL, provider, and per-task route models for chat, audio transcript, and image
extraction. Environment variables remain operational fallback inputs for
legacy/unconfigured fields, but source defaults and root TOML are the durable
configuration surfaces.

The live vLLM-SR acquisition client uses the official OpenAI-compatible data
plane as a route probe: `POST /v1/chat/completions` with `model: auto`.
`xiuxian-llm` parses the resulting vLLM-SR decision headers, including
`x-vsr-selected-model`, `x-vsr-selected-decision`,
`x-vsr-selected-confidence`, `x-vsr-selected-reasoning`, and
`x-vsr-selected-modality`, into a `WendaoModelDecision`. The selected Wendao
backend profile remains a Gateway execution concern; the analyzer only receives
the final metadata.

Local developer runs default to Gateway-owned `deterministic` routing so the
pure local experience does not require Docker or Kubernetes. The deterministic
policy still returns a `WendaoModelDecision`; frontends and adapters must
consume the Gateway decision rather than selecting a model locally.

When `WENDAO_MODEL_ROUTING_MODE=vllm-sr` is configured, the process-managed
vLLM-SR sidecar validates its config before serve and uses the upstream
`vllm-sr serve` runtime. Docker or Kubernetes target settings are deployment
concerns for that mode. Missing Docker, a Podman-backed Docker shim, or an
unreachable Docker daemon is an infrastructure admission failure in vLLM-SR
mode, not a reason to silently change routing mode.

The first Gateway consumers are Studio document-extract audio shards and
standalone image VLM extraction. Studio uses the shared attachment route helpers
for both deterministic and vLLM-SR modes, then sends selected
provider/model/backend metadata on the existing audio shard Flight exchange or
primary document-extract Flight route. This keeps `xiuxian-llm` as the contract
owner while leaving scheduling, artifact identity, and precision gates in
Studio and Wendao attachment crates.

Chat uses the same contract shape through `xiuxian-llm` chat route helpers.
They build `taskKind=chat`, `modality=text`, and `sourceKind=conversation`
intents, admit them through vLLM-SR when that mode is active, and apply the
selected provider/model decision to the existing OpenAI-compatible runtime
profile resolver. The resolver still owns endpoint, credential, and wire-mode
lookup for the selected provider; callers should not hardcode chat model
selection in frontend or adapter code.
