# OpenAI-Compatible Semantic Ignition

:PROPERTIES:
:ID: feat-openai-semantic-ignition
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, retrieval, quantum-fusion, arrow, openai-compatible
:STATUS: ACTIVE
:VERSION: 1.0
:END:

## Overview

`OpenAiCompatibleSemanticIgnition` extends Wendao's hybrid retrieval ignition
layer to accept query text and resolve vectors from an OpenAI-compatible
`/v1/embeddings` endpoint before searching the Lance vector-store facade.

The adapter is designed for real gateway environments (for example local GLM
gateway deployments) without changing the Arrow-native fusion and scoring
pipeline.

## Architecture Position

1. Input: `QuantumSemanticSearchRequest`.
2. Query vector resolution:
   - Use `query_vector` directly when provided.
   - Else call OpenAI-compatible embedding transport with `query_text`.
3. Vector retrieval: call `VectorStore::search_optimized`.
4. Fusion: pass anchors into existing quantum orchestration and Arrow scoring.

## Runtime Notes

- The adapter is additive and does not replace `VectorStoreSemanticIgnition`.
- Telemetry reports the storage detail as `lance-vector-store` instead of the
  retiring `xiuxian-vector` crate name.
- Authentication can be injected by supplying a custom `reqwest::Client` with
  default headers through `with_embedding_client(...)`.
- The embedding endpoint base URL is normalized through
  `xiuxian_llm::embedding::openai_compat`.

## Runtime Activation

Enable the runtime wiring through `link_graph.retrieval.semantic_ignition`:

```toml
[link_graph.retrieval]
mode = "hybrid"
candidate_multiplier = 4
max_sources = 8
hybrid_min_hits = 2
hybrid_min_top_score = 0.25
graph_rows_per_source = 8

[link_graph.retrieval.semantic_ignition]
backend = "glm"
vector_store_path = ".cache/wendao/vector-store"
table_name = "wendao_semantic_docs"
embedding_base_url = "http://127.0.0.1:11434"
embedding_model = "glm-5"
```

- `backend = "glm"` resolves to the OpenAI-compatible ignition path.
- `backend = "vector_store"` reuses precomputed vectors without embedding calls.
- `backend = "disabled"` keeps planned search on the graph-only path.

The same runtime can also be supplied through environment variables:

- `XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_BACKEND`
- `XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_VECTOR_STORE_PATH`
- `XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_TABLE_NAME`
- `XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_EMBEDDING_BASE_URL`
- `XIUXIAN_WENDAO_LINK_GRAPH_SEMANTIC_IGNITION_EMBEDDING_MODEL`

## Surfaced Outputs

- `LinkGraphPlannedSearchPayload` now exposes `semantic_ignition` telemetry and
  the resolved `quantum_contexts`.
- `zhenfa_router` markdown and XML-Lite output surfaces the ignition backend,
  context count, and any degradation error.
- Failures are non-fatal: graph hits remain available and telemetry records the
  semantic-ignition error instead of aborting planned retrieval.

## Query Contract

Text-only semantic queries are now treated as valid ignition input. When
`query_text` is present, Wendao resolves embeddings first and no longer drops
the request just because `query_vector` is empty.

## Validation Target

- `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::runtime_config::tests::`
- `CARGO_TARGET_DIR=/tmp/xiuxian-artisan-workshop-codex-semantic-ignition-target direnv exec . cargo test -p xiuxian-wendao --test planned_search_semantic_ignition`
- `CARGO_TARGET_DIR=/tmp/xiuxian-artisan-workshop-codex-semantic-ignition-target direnv exec . cargo test -p xiuxian-wendao --test quantum_fusion_openai_ignition`
- `CARGO_TARGET_DIR=/tmp/xiuxian-artisan-workshop-codex-semantic-ignition-target direnv exec . cargo clippy -p xiuxian-wendao --tests -- -D warnings`

:RELATIONS:
:LINKS: [[03_features/203_agentic_navigation|Agentic Navigation (wendao.agentic_nav)]], [[03_features/205_semantic_auditor|Semantic Auditor (wendao audit)]]
:END:
