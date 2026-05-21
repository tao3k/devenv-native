# Wendao Knowledge Retrieval Benchmark

`wendao-knowledge-retrieval-benchmark` is the Python-owned black-box benchmark
harness for Wendao knowledge retrieval profiles.

The package reads existing `xiuxian_wendao.real_repo_search_precision.v1`
receipts and compares retrieval strategies without changing Rust runtime
routes, Julia services, Arrow schemas, or search-ranking behavior.

The first supported profiles are:

- `flat-topk`: estimates the cost of exposing all observed top-k result paths
  from linked query receipts.
- `graph-first-reasoning-tree`: estimates progressive disclosure cost from
  scenario reasoning-tree receipts, including anchors, semantic relation hops,
  PageIndex seed evidence, source evidence, and disclosure depth.
- `intent-tree-v1`: estimates an agent-facing reasoning-tree profile from
  deterministic scenario intent frames plus the existing reasoning-tree
  receipts. It measures the extra intent parsing, evidence coverage, and
  verifier scaffolding cost for external orchestration layers.
- `backend-frontier-pruning-v1`: consumes Rust-emitted backend frontier nodes
  when present. It measures the first Rust-control-plane pruning contract that
  can later receive WendaoGraph.jl graph scores and pi-wendao subagent
  judgements. Current Julia schedule actions are static warm-profile
  projections, not live Julia execution evidence.
- `search-strategy-flow-projection-v1`: consumes the receipt-derived
  SearchStrategyFlow candidate, transition, and selected-frontier projection.
  It compares the graph contract as an independent black-box strategy without
  claiming live WendaoGraph.jl execution.

Profile rows also report intent-evidence coverage and context-reduction
diagnostics so downstream agent/workflow systems can compare progressive
disclosure strategies without using the benchmark package as a live router.
Backend-frontier rows additionally expose subagent fanout counts as
`fanout/group/max-width` and Julia scheduling projection counts as
`candidate/dispatch/queue/fallback/reject`.
SearchStrategyFlow rows expose candidate, transition, and selected-frontier
projection counts as `candidate/transition/frontier`.
Validation guards such as authority ordering and negative evidence can appear
as candidate/transition facts without becoming selected frontier branches.
They also report `Flow Loop/LLM` as
`loop-budget/cycle-candidate-nodes/llm-judgement-nodes`, because
SearchStrategyFlow is a strategy topology for iterative graph reasoning, not a
single-pass retrieval algorithm.
Each profile row includes per-scenario diagnostics so callers can identify
which natural-language intent still lacks relation, PageIndex, authority, or
source-path evidence. Scenario diagnostics also expose SearchStrategyFlow
topology, candidate/transition/frontier counts, and loop/LLM counts so an
external agent can prune by intent instead of only by repository aggregate.
The report also emits additive scenario-level recommendations, allowing simple
known-item lookups to stay on `flat-topk` while evidence-rich scenarios can
select graph-first, intent-tree, or SearchStrategyFlow profiles. When quality
and context cost are otherwise tied, SearchStrategyFlow topology facts can
break the tie only for scenarios with iterative or cyclic refinement evidence.
Repository-level recommendations summarize those scenario decisions instead of
promoting a topology-heavy profile across a flat exact-lookup workload.

The benchmark accepts multi-repository receipts. This includes receipts where
Wendao backend evidence and external orchestration evidence, such as
[`pi-wendao`](https://github.com/tao3k/pi-wendao), appear as separate
repository rows. The package still compares profile evidence only; it does not
run subagents or call an LLM.

Future slices can add optional Julia-backed PPR, community-frontier, and hybrid
profiles as measured backends. Python remains the benchmark judge; Rust and
Julia remain benchmarked implementations.

The package is managed by `python-lang-project-harness` through its local
project configuration and package-level harness test.

## Documentation

- [Package docs](docs/README.md)
- [Architecture](docs/architecture.md)
- [Profile contract](docs/profile_contract.md)
- [Usage](docs/usage.md)

## Usage

```bash
wendao-knowledge-retrieval-benchmark \
  --receipt path/to/real_repo_receipt.json \
  --output-json path/to/knowledge_retrieval_benchmark.json \
  --output-markdown path/to/knowledge_retrieval_benchmark.md
```
