---
type: knowledge
title: "Audit: Polyglot Compute Orchestrator"
category: "audit"
status: "draft"
authors:
  - codex
created: 2026-05-04
tags:
  - audit
  - orchestrator
  - wendao
  - polyglot
  - python
  - julia
metadata:
  title: "Audit: Polyglot Compute Orchestrator"
---

# Audit: Polyglot Compute Orchestrator

- **RFC Reference**: [2026-05-04-polyglot-compute-orchestrator-rfc.md](./2026-05-04-polyglot-compute-orchestrator-rfc.md)
- **Status**: Calibrated draft audit
- **Authority**: Recommendation only; this audit does not approve physical
  implementation.

## 1. Executive Summary

This audit evaluates the architectural proposal for a polyglot compute
coordination lane across Rust, Python/Docling, and Julia.

The direction is valuable, but the first draft over-centralized the design into
a new crate and overclaimed several transport, schema, and scheduling
properties. The corrected architecture should extend existing Wendao runtime,
document extraction, attachments, analyzer, and Julia plugin boundaries before
introducing a standalone orchestration crate.

## 2. Evidence Calibration

### 2.1 Existing Python Document Extraction Boundary

The worktree already has a Python analyzer Flight service:

1. `DocumentExtractFlightServer` exposes `/analysis/document-extract` and
   `/analysis/pdf-ocr-shards`.
2. `xiuxian-wendao-runtime` defines the stable document extraction route and
   metadata contract.
3. `xiuxian-wendao-attachments` documents Rust-owned OCR shard scheduling,
   ordering validation, cache policy, and Docling fallback authority.

Design implication: the first implementation must reuse these boundaries rather
than introduce a second Docling wrapper or a second document-extraction
scheduler.

### 2.2 Existing Julia Compute Boundary

The worktree already has staged Julia compute contracts:

1. `memory.julia_compute` runtime config carries `timeout_secs`,
   `max_in_flight_requests`, fallback, routes, schema version, and profile
   identity.
2. `xiuxian-wendao-julia` owns family/profile contracts for episodic recall,
   gate scoring, plan tuning, and calibration.
3. Existing graph-structural routes and the WendaoGraph evidence lane frame
   Julia as advisory compute, not storage or authority.

Design implication: Rust may gate requests by profile and route, but it cannot
claim task stealing onto Julia internal threads unless a new Julia worker-queue
protocol is approved.

### 2.3 External Evidence

The following sources support narrower claims only:

1. Docling's model catalog identifies Heron as the default layout model and
   lists supported engines, but it also warns that performance varies by
   hardware, document complexity, and model size.
2. The Arrow PyCapsule Interface standardizes in-process Python export of Arrow
   C Data Interface capsules. It does not define cross-process shared-memory
   transport.
3. HTTP/2 defines the initial stream and connection flow-control window as
   65,535 octets and the default max frame size as 16,384 octets. The earlier
   32 KiB window claim was incorrect.
4. ThreadPinning.jl supports Julia-side thread pinning. It does not imply Rust
   owns Julia worker-thread scheduling.
5. OhMyThreads.jl supports task-based Julia data-parallel compute. It does not
   define a cross-language work-stealing protocol.

## 3. Required Corrections

1. **Zero-copy transport**: downgrade to copy-aware Arrow Flight/IPC transport.
   Treat true cross-process zero-copy as a future shared-memory pilot requiring
   allocator, descriptor, lifetime, cleanup, crash-recovery, and container
   boundary contracts.
2. **Package boundary**: replace immediate new-crate creation with first-slice
   extensions to existing runtime, provider, attachments, analyzer, and Julia
   plugin owners.
3. **Schema strategy**: demote the global super-schema from mandatory strategy
   to benchmark candidate beside profile-specific, normalized long-table, and
   nested schema options.
4. **Transport tuning**: replace the 32 KiB/4 MiB gRPC claim with measured
   Flight data-path tuning for Tonic message sizes, stream windows, connection
   windows, and batch sizing.
5. **Julia scheduling**: replace task stealing with Rust-side admission control
   and Julia-side scheduling telemetry.
6. **SSOT dependency**: remove semantic SSOT routing as a dependency. It can
   become a later consumer after separate approval.

## 4. Final Verdict: Revision Required

The RFC is directionally useful but should not be treated as implementation
authorization. It becomes ready for Sovereign review only after the required
corrections are reflected in the RFC and the first rollout is bounded to
existing physical contracts.

Physical implementation remains blocked for:

1. a new `xiuxian-polyglot-orchestrator` crate
2. a new Python Docling wrapper service
3. shared-memory zero-copy transport
4. global super-schema adoption
5. Rust-to-Julia task stealing
6. semantic SSOT-based routing

## 5. References

1. [Docling Model Catalog](https://docling-project.github.io/docling/usage/model_catalog/)
2. [Apache Arrow PyCapsule Interface](https://arrow.apache.org/docs/format/CDataInterface/PyCapsuleInterface.html)
3. [RFC 9113: HTTP/2](https://www.rfc-editor.org/rfc/rfc9113.html)
4. [ThreadPinning.jl: Pinning Julia Threads](https://carstenbauer.github.io/ThreadPinning.jl/v0.7.2/examples/ex_pinning_julia_threads/)
5. [OhMyThreads.jl](https://juliafolds2.github.io/OhMyThreads.jl/stable/)
