# xiuxian-graph-core

`xiuxian-graph-core` owns reusable graph projection primitives and optional
rendering or graph-algorithm adapters.

## Ownership Boundary

- This crate owns generic `GraphProjection`, `GraphNode`, `GraphEdge`, and
  `GraphNodeId` data structures.
- Domain crates map local semantics into those primitives at their own
  boundary. For example,
  [`xiuxian-julia-core`](../xiuxian-julia-core/README.md) maps
  `WendaoGraph.jl` readiness and schedule facts into graph projections.
- Optional features keep heavier adapters explicit:
  - `mermaid` enables compact Mermaid rendering and `merman-core` validation.
    Domain adapters should provide Mermaid-safe node ids and keep source-domain
    identifiers in labels when those identifiers contain punctuation.
  - `petgraph` enables conversion to stable `petgraph` graphs.

## Non-Goals

- This crate does not own Julia, Wendao, Qianji, Org, SDD, or runtime
  scheduling semantics.
- This crate does not start services, read project config, open Flight routes,
  or decide admission policy.
