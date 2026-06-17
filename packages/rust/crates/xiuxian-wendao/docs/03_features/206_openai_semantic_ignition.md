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
layer for vector-store backed semantic search. Default Wendao builds use
precomputed query vectors only. In-process OpenAI-compatible `/v1/embeddings`
transport is a compatibility path behind the explicit `llm` Cargo feature.

The adapter is designed for gateway environments where embedding execution is
usually owned by an external Agent or model service. Enabling `llm` keeps a
legacy local transport path available without making provider execution part of
the default Wendao boundary.

## Architecture Position

1. Input: `QuantumSemanticSearchRequest`.
2. Query vector resolution:
   - Use `query_vector` directly when provided.
   - Else, when the explicit `llm` feature is enabled, call
     OpenAI-compatible embedding transport with `query_text`.
   - Else reject text-only input with a provider-disabled error so callers send
     a precomputed vector or route embedding work to an external service.
3. Vector retrieval: call `VectorStore::search_optimized`.
4. Fusion: pass anchors into existing quantum orchestration and Arrow scoring.

## Runtime Notes

- The adapter is additive and does not replace `VectorStoreSemanticIgnition`.
- Telemetry reports the storage detail as `lance-vector-store` instead of the
  retiring `xiuxian-vector` crate name.
- Authentication can be injected by supplying a custom `reqwest::Client` with
  default headers through `with_embedding_client(...)` when `llm` is enabled.
- The embedding endpoint base URL is normalized through
  `xiuxian_llm::embedding::openai_compat` only in that explicit compatibility
  build.

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

Default semantic queries must provide `query_vector`. Text-only semantic
queries are accepted only in explicit `llm` compatibility builds, where Wendao
resolves embeddings before vector search. Without `llm`, text-only queries fail
closed with `EmbeddingProviderDisabled`.

## Validation Target

- `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::runtime_config::tests::`
- `direnv exec . cargo test -p xiuxian-wendao --test quantum_fusion_openai_ignition --features vector-store`
- `direnv exec . cargo test -p xiuxian-wendao --test quantum_fusion_openai_ignition --features "vector-store llm"`
- `direnv exec . cargo clippy -p xiuxian-wendao --tests --features vector-store -- -D warnings`

:RELATIONS:
:LINKS: [[03_features/203_agentic_navigation|Agentic Navigation (wendao.agentic_nav)]], [[03_features/205_semantic_auditor|Semantic Auditor (wendao audit)]]
:END:
