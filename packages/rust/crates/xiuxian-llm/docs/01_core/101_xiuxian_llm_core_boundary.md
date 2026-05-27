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

The live vLLM-SR acquisition client uses the official OpenAI-compatible data
plane as a route probe: `POST /v1/chat/completions` with `model: auto`.
`xiuxian-llm` parses the resulting vLLM-SR decision headers, including
`x-vsr-selected-model`, `x-vsr-selected-decision`,
`x-vsr-selected-confidence`, `x-vsr-selected-reasoning`, and
`x-vsr-selected-modality`, into a `WendaoModelDecision`. The selected Wendao
backend profile remains a Gateway execution concern; the analyzer only receives
the final metadata.

The first Gateway consumers are Studio document-extract audio shards and
standalone image VLM extraction. Studio uses the shared routing mode and vLLM-SR
base URL helpers, then sends selected provider/model/backend metadata on the
existing audio shard Flight exchange or primary document-extract Flight route.
This keeps `xiuxian-llm` as the contract owner while leaving scheduling,
artifact identity, and precision gates in Studio and Wendao attachment crates.

Chat uses the same contract shape through `xiuxian-llm` chat route helpers.
They build `taskKind=chat`, `modality=text`, and `sourceKind=conversation`
intents, admit them through vLLM-SR when that mode is active, and apply the
selected provider/model decision to the existing OpenAI-compatible runtime
profile resolver. The resolver still owns endpoint, credential, and wire-mode
lookup for the selected provider; callers should not hardcode chat model
selection in frontend or adapter code.
