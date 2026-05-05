---
type: knowledge
kind: audit
title: "Audit: Polyglot Compute Orchestrator"
category: "audit"
status: "live-backend-probe-profile-complete"
author: Xiuxian Artisan Workshop
authors:
  - codex
created: 2026-05-04
date: 2026-05-05T00:00-07:00
description: "Audit of the bounded polyglot compute orchestrator crate and its preserved execution-owner boundaries."
tags:
  - audit
  - orchestrator
  - wendao
  - polyglot
  - python
  - julia
metadata:
  title: "Audit: Polyglot Compute Orchestrator"
  retrieval:
    saliency_base: 7.0
    decay_rate: 0.03
---

# Audit: Polyglot Compute Orchestrator

:PROPERTIES:
:ID: audit-polyglot-compute-orchestrator
:END:

- **RFC Reference**: [2026-05-04-polyglot-compute-orchestrator-rfc.md](./2026-05-04-polyglot-compute-orchestrator-rfc.md)
- **Status**: Live backend probe profile complete
- **Authority**: This audit approves a thin
  `xiuxian-polyglot-orchestrator` crate boundary for shared Rust control-plane
  contracts plus the completed inert pressure-evidence and Julia
  readiness-evidence slices, plus the completed inert schema-benchmark evidence
  and report-contract slices, plus the completed pure Docling scheduling-plan
  contract and owner-adoption slices, plus the completed cross-chain harness
  profile slice, plus the completed full-document runtime-adoption slice, plus
  the completed backend profile optimization slice, plus the completed live
  backend probe profile slice. It does not approve Python public API, schema,
  route, worker lifecycle, shared-memory, semantic routing, or later rollout
  implementation.

## 1. Executive Summary

:PROPERTIES:
:ID: 1-executive-summary
:END:

This audit evaluates the architectural proposal for a polyglot compute
coordination lane across Rust, Python/Docling, and Julia.

The direction is valuable, but the first draft overcentralized the design and
overclaimed several transport, schema, and scheduling properties. The corrected
architecture should introduce a dedicated crate only as a thin Rust
control-plane contract boundary while preserving the existing Wendao runtime,
document extraction, attachments, analyzer, and Julia plugin owners.

## 2. Evidence Calibration

:PROPERTIES:
:ID: 2-evidence-calibration
:END:

### 2.1 Existing Python Document Extraction Boundary

The worktree already has a Python analyzer Flight service:

1. `DocumentExtractFlightServer` exposes `/analysis/document-extract` and
   `/analysis/pdf-ocr-shards`.
2. `xiuxian-wendao-runtime` defines the stable document extraction route and
   metadata contract.
3. `xiuxian-wendao-attachments` documents Rust-owned OCR shard scheduling,
   ordering validation, cache policy, and Docling fallback authority.

Design implication: implementation must reuse these boundaries rather than
introduce a second Docling wrapper or a second live document-extraction
scheduler. A pure Rust scheduling-plan contract is allowed only when owner
crates still execute the plan through their existing routes, headers, batches,
caches, ordering validation, and fallback policy.

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

:PROPERTIES:
:ID: 3-required-corrections
:END:

1. **Zero-copy transport**: downgrade to copy-aware Arrow Flight/IPC transport.
   Treat true cross-process zero-copy as a future shared-memory pilot requiring
   allocator, descriptor, lifetime, cleanup, crash-recovery, and container
   boundary contracts.
2. **Package boundary**: approve `xiuxian-polyglot-orchestrator` only as a
   shared Rust control-plane contract crate. Existing runtime, provider,
   attachments, analyzer, and Julia plugin owners remain the execution and
   schema authorities.
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

## 4. Final Verdict: Live Backend Probe Profile Complete

:PROPERTIES:
:ID: 4-final-verdict-live-backend-probe-profile-complete
:END:

The RFC is directionally useful and the required corrections are reflected
well enough to authorize the completed crate-boundary and pressure-evidence
slices. The approved crate remains a thin contract boundary for lane identity,
admission, readiness, pressure, fallback, route/profile references, and inert
control-plane snapshots.

The completed Phase 2 slice records Python Docling and OCR pressure facts from
owner-supplied counters only. The completed Phase 3 slice records Julia
profile, route, schema, manifest, warmup, benchmark, and admission-window facts
from owner-supplied evidence only. The completed Phase 4 slice records
schema-strategy benchmark observations from owner-supplied evidence only and
adds report contracts for aggregating those observations without approving any
schema default. The completed Docling scheduling slice adds pure scheduling
plans that map owner-supplied pressure evidence and caller-local worker or
shard bounds to `dispatch`, `queue`, `fallback`, or `reject`. Owner crates still
translate those plans into existing route, header, batch, cache, ordering, and
fallback behavior. The completed owner-adoption slice routes Studio's common
OCR worker/shard clamp through that plan after local adaptive pressure and
source-range ceiling policy are computed. Studio still owns live semaphores,
queue wait observation, endpoint dispatch, cache, in-flight coalescing, and the
worker-budget Flight header. The completed harness-profile slice makes the
current chain directly auditable through `rust-lang-project-harness`: the
orchestrator crate self-applies the gate, keeps module interfaces
interface-only, mounts unit tests from `tests/unit/lib`, and records profile
hints for the control-plane and Docling scheduler contracts. Runtime,
attachments, Julia, and Studio owner surfaces also expose focused profile
hints for the polyglot bridge or adoption points. The completed
full-document runtime-adoption slice routes Studio's existing full-document
Docling dispatch through the runtime-owned inert schedule plan before endpoint
selection. That adoption uses the existing conversion semaphore as the owner
budget and preserves endpoint-pool routing, cache/job registry behavior,
Python Flight routes, schemas, and worker lifecycle. The completed backend
profile optimization slice runs focused Rust/Python backend tests with timing
evidence and fixes profile drift in the attachment and Julia owner surfaces:
the attachment bridge profile now records the required `pdf-source-range`
regression command, and Julia package docs point to the actual lib-mounted
readiness test target. The completed live backend probe profile slice runs the
existing analyzer/Studio background provider benchmark, fixes harness drift in
that path, and adds an executable OCR-positive PDF milestone guard. The guard
uses the Rust scheduler's automatic source-range worker policy as the default
gate; fixed source-range worker counts remain diagnostic overrides only. Later
Phase 2 runtime control work beyond this adoption, later Phase 3 live readiness
work, later Phase 4 live benchmark or schema selection work, and Phase 5 remain
unauthorized until separate scoped ExecPlans define their implementation and
validation gates.

Physical implementation is complete for:

1. a new `xiuxian-polyglot-orchestrator` workspace crate
2. crate README and package-boundary documentation
3. lane/admission/evidence/reference contracts that do not change runtime
   behavior or existing public Python/Julia routes
4. read-only runtime, attachments, and Julia bridge helpers
5. inert control-plane snapshots
6. pressure evidence for supplied document-extraction and OCR-shard counters
7. readiness evidence for supplied Julia memory-family profile facts
8. schema benchmark evidence for supplied schema-strategy observations
9. schema benchmark report contracts for supplied evidence rows
10. pure Docling scheduling-plan contracts for supplied pressure facts
11. Studio owner adoption for the common OCR worker/shard clamp
12. rust-lang-project-harness profile coverage for the orchestrator crate,
    owner polyglot bridges, and Studio adoption point
13. Studio full-document runtime adoption for the existing Docling endpoint
    dispatch path
14. backend profile validation evidence for the orchestrator/runtime/
    attachment/Julia/Studio/analyzer chain, plus corrected attachment and Julia
    owner profile commands/docs
15. live background document-extract probe evidence and the stored
    OCR-positive PDF milestone regression guard using Rust auto scheduling

Physical implementation remains blocked for:

1. crate-owned Python Docling execution or worker lifecycle
2. a new Python Docling wrapper service
3. route, schema, or public Python API changes
4. shared-memory zero-copy transport
5. global super-schema adoption
6. Rust-to-Julia task stealing
7. semantic SSOT-based routing

## 5. References

:PROPERTIES:
:ID: 5-references
:END:

1. [Docling Model Catalog](https://docling-project.github.io/docling/usage/model_catalog/)
2. [Apache Arrow PyCapsule Interface](https://arrow.apache.org/docs/format/CDataInterface/PyCapsuleInterface.html)
3. [RFC 9113: HTTP/2](https://www.rfc-editor.org/rfc/rfc9113.html)
4. [ThreadPinning.jl: Pinning Julia Threads](https://carstenbauer.github.io/ThreadPinning.jl/v0.7.2/examples/ex_pinning_julia_threads/)
5. [OhMyThreads.jl](https://juliafolds2.github.io/OhMyThreads.jl/stable/)
