# Architecture

`wendao-knowledge-retrieval-benchmark` is a Python package for black-box
retrieval profile comparison. It does not own Wendao indexing, ranking,
runtime routes, Julia execution, or Arrow schemas. Its job is to evaluate
evidence that those systems already produced.

## Ownership

The package owns:

- loading `xiuxian_wendao.real_repo_search_precision.v1` receipts;
- computing comparable profile rows from those receipts;
- rendering JSON and Markdown benchmark reports;
- package-level validation through `python-lang-project-harness`.

Rust owns:

- real-repository catalog and precision receipt generation;
- LinkGraph indexing and search ranking;
- semantic/PageIndex evidence extraction before any optional Julia dispatch.

Julia owns future measured implementations such as PPR-like relatedness,
community-frontier exploration, and large graph traversal. This package may
score those implementations only after their outputs appear as explicit
profile inputs.

## Data Flow

1. A Wendao precision run emits a real-repository receipt.
2. The benchmark CLI reads that receipt from a caller-provided path.
3. Profile scorers compute comparable `ProfileScore` rows.
4. The report renderer writes JSON and Markdown to caller-provided paths, or
   prints Markdown to stdout.

The package is deliberately offline and deterministic once the receipt exists.
It does not refresh repositories, start services, call Julia, or query a live
search endpoint.
The Markdown report includes aggregate profile rows, per-scenario
recommendations, and per-scenario diagnostic rows. The scenario rows are
intended for external orchestrators that need to decide which backend evidence
to disclose next without reading raw Wendao internals. SearchStrategyFlow
scenario diagnostics include the intent complexity class, initial and
refinement topology, candidate/transition/frontier counts, and loop/LLM counts,
which makes cyclic or iterative graph reasoning visible at the scenario level.
Receipts may contain multiple repositories. For example, Wendao backend
knowledge and [`pi-wendao`](https://github.com/tao3k/pi-wendao) orchestration
evidence can be measured as separate repository rows while preserving package
ownership: Rust emits evidence, Python scores profiles, and the external
orchestrator owns real LLM/subagent execution.

## Current Profiles

`flat-topk` estimates the cost of exposing all observed top-k paths linked to
the scenario queries.

`graph-first-reasoning-tree` estimates progressive disclosure over scenario
reasoning-tree steps: anchor query, semantic relation, PageIndex seed, and
source evidence.

`intent-tree-v1` estimates the next agent-facing search profile. It combines
deterministic scenario intent frames with reasoning-tree receipts so the
benchmark can measure intent anchoring, required evidence coverage, relation
hypotheses, authority policy, verifier scaffolding, context reduction, and
progressive disclosure depth for external agent/workflow orchestration layers.

`backend-frontier-pruning-v1` consumes Rust-emitted backend frontier nodes when
present. This is the first benchmark surface for the Rust control plane,
future WendaoGraph.jl graph scores, and pi-wendao subagent branch judgements.
In this slice it is receipt-derived and may use `rust-baseline` as the graph
backend. Julia schedule actions are treated as static projections until real
WendaoGraph readiness, queue, and latency evidence is attached.
The report renders these projections as `Julia C/D/Q/F/R`, where the columns
mean candidate, dispatch, queue, fallback, and reject counts.
It also renders `Agent F/G/W` for backend-frontier rows, where the columns mean
subagent fanout nodes, fanout groups, and maximum parallel width.
`Flow C/T/F` is the SearchStrategyFlow receipt projection: candidate rows,
transition rows, and selected frontier rows. It is a graph-contract diagnostic,
not proof that a live WendaoGraph.jl route executed.

`search-strategy-flow-projection-v1` is the independent benchmark profile for
those same projection fields. It scores only the selected frontier rows by
their SearchStrategyFlow rank and materialized context cost; projected context
budget remains a diagnostic field for later subagent execution. This lets the
benchmark compare the graph contract directly against backend-frontier pruning
without changing runtime retrieval behavior.
Authority and negative-guard rows remain candidate and transition facts, but
they are validation guards rather than selected frontier branches.
The report also renders `Flow Loop/LLM`, where the columns mean loop budget,
cycle-candidate nodes, and LLM/subagent judgement nodes. These are topology
planning facts: the first pass remains an acyclic evidence DAG, while later
relation/PageIndex refinement can become cyclic only when the receipt marks
revisitable graph nodes or subagent judgement surfaces.
Scenario recommendations use those topology facts as a bounded tie-breaker:
SearchStrategyFlow may beat graph-first only when the evidence and cost scores
are already competitive and the scenario carries iterative or cyclic refinement
evidence.
The repository recommendation then aggregates scenario winners, which keeps
flat exact-lookup repositories on the cheap path even if an aggregate
SearchStrategyFlow profile has diagnostic topology fields.

The first real comparison keeps quality equal while reducing exposed
path-character cost:

| Profile                              | Scenarios | Recall@10 |  MRR | Evidence | Context cut | Exposed chars | Steps | Max depth |
| ------------------------------------ | --------: | --------: | ---: | -------: | ----------: | ------------: | ----: | --------: |
| `flat-topk`                          |       7/7 |     10000 | 9285 |     7/17 |           0 |         13777 |     0 |         0 |
| `graph-first-reasoning-tree`         |       7/7 |     10000 | 9285 |    17/17 |        7023 |          4101 |    31 |         2 |
| `intent-tree-v1`                     |       7/7 |     10000 | 9285 |    17/17 |        5831 |          5743 |    31 |         2 |
| `backend-frontier-pruning-v1`        |       7/7 |     10000 | 9285 |    17/17 |        6914 |          4251 |    33 |         2 |
| `search-strategy-flow-projection-v1` |       7/7 |     10000 | 9285 |    17/17 |        7023 |          4101 |    31 |         2 |

## Non-Goals

This package does not:

- implement a search algorithm;
- implement LLM orchestration or sub-agent execution;
- change Rust or Julia runtime behavior;
- replace the Rust precision harness;
- treat speculative Julia profiles as accepted evidence;
- make hidden operational paths part of canonical docs.
