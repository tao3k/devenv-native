---
type: knowledge
kind: readme
title: "xiuxian-polyglot-orchestrator"
category: "package-docs"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Package README for the bounded Wendao polyglot compute orchestrator control-plane contracts."
tags:
  - orchestrator
  - wendao
  - polyglot
  - docling
metadata:
  title: "xiuxian-polyglot-orchestrator"
  retrieval:
    saliency_base: 7.0
    decay_rate: 0.03
---

# xiuxian-polyglot-orchestrator

Thin Rust control-plane contracts for the Wendao polyglot compute lane.

This crate is governed by
[RFC: Polyglot Compute Orchestrator](../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md)
and its
[audit](../../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md).

## Ownership Boundary

`xiuxian-polyglot-orchestrator` owns only shared Rust contracts:

1. lane identity for Python Docling and Julia compute
2. admission budgets and decisions
3. health, readiness, pressure, and fallback evidence envelopes
4. typed references to existing route, profile, and schema owners
5. worker-pressure evidence derived from owner-supplied counters
6. Julia readiness evidence derived from owner-supplied profile facts
7. schema benchmark evidence and reports derived from owner-supplied
   observations
8. pure Docling scheduling plans derived from owner-supplied pressure evidence
   and caller-local worker or shard bounds
9. pure Julia profile scheduling plans derived from owner-supplied readiness,
   runtime statistics, task shape evidence, fallback availability, and latency
   constraints
10. Julia thread topology and thread-pinning diagnostics as evidence-only
    readiness facts supplied by Julia owner packages
11. Julia accelerator/backend diagnostics as evidence-only readiness facts
    supplied by Julia owner packages

It does not own Python Docling execution, OCR shard ordering, document cache
policy, Julia profile schemas, Julia JIT warmup execution, Julia thread
scheduling, `ThreadPinning.jl` policy application, accelerator selection or
device execution, Arrow Flight transport
construction, schema default selection, shared-memory transport, or semantic
routing. Scheduling plans are advisory contracts; owner packages still execute
or decline work through their existing routes, headers, queues, caches, and
fallback policy. Rust may select a Julia pod and trigger owner-exposed prewarm
routes before release, but Julia remains responsible for internal warmup,
thread pinning, queues, and numerical execution. The Studio OCR scheduler
supplies live pressure and system facts, then consumes these plans for
source-range auto worker sizing and the common worker/shard clamp. The Studio
full-document provider also consumes the runtime-owned plan before existing
Docling endpoint selection while retaining endpoint-pool, cache/job registry,
and Python worker lifecycle authority.
The source-range auto policy sizes PDF page-range OCR waves from the supplied
adaptive budget, machine-derived cap, remaining permits, shard count, and a
conservative target of seven source pages per worker; fixed source-range worker
values remain diagnostic overrides, not production defaults.

## Existing Owners

1. `xiuxian-wendao-runtime` owns deployment config, Flight transport substrate,
   route-level request gates, timeout policy, schema metadata, and translation
   of document-extraction plans into runtime behavior.
2. `xiuxian-wendao-attachments` owns OCR shard scheduling evidence, cache
   reuse, ordering validation, Docling fallback policy, and translation of OCR
   shard plans into attachment-local batches.
3. `xiuxian-wendao-julia` owns Julia profile, schema, manifest, route
   validation, warmup, benchmark, thread diagnostics, and readiness contracts.
4. `xiuxian-wendao-analyzer` owns Python document conversion and OCR execution
   behind the existing analyzer Flight service.

## Bootstrap Modules

1. `lanes`: polyglot lane identity and capability classification.
2. `admission`: reusable admission budget and decision contracts.
3. `evidence`: health, readiness, pressure, and fallback evidence.
4. `pressure`: worker budget, queue, failure, and ordering pressure evidence.
5. `docling_schedule`: inert document-extraction and OCR-shard scheduling
   plans derived from supplied pressure facts.
6. `julia_schedule`: inert Julia dispatch, queue, fallback, and reject plans
   derived from supplied profile evidence and task shape facts.
7. `readiness`: Julia profile, route, schema, manifest, warmup, benchmark,
   thread-diagnostic, and accelerator-diagnostic readiness evidence.
8. `schema_benchmark`: advisory schema-strategy benchmark evidence and report
   contracts.
9. `refs`: typed references to external owner contracts.
10. `snapshot`: inert read-only aggregation of refs, admission budgets, and
    evidence.

## Project Policy Gate

This crate self-applies `rust-lang-project-harness` from `src/lib.rs`. The
active profile marks the crate root as the shared control-plane public API and
the Docling and Julia scheduler implementations as owner-facing scheduling
contracts.
Module `mod.rs` files stay interface-only and re-export leaf `model.rs`
implementation files. Unit tests live under `tests/unit/lib` and are mounted
from the crate root so `cargo test --lib` runs both behavior tests and the
harness gate.

The backend profile evidence chain for this lane is intentionally focused:
`cargo test -p xiuxian-polyglot-orchestrator --all-features`,
`cargo test -p xiuxian-wendao-runtime --lib polyglot`,
`cargo test -p xiuxian-wendao-attachments --features pdf-source-range --lib polyglot`,
`cargo test -p xiuxian-wendao-julia --lib polyglot`, the Studio
`document_extract` lib tests with document-extract source-range features, and
the analyzer document-service pytest suite. These tests verify profile and
owner-boundary behavior; they do not authorize new routes, schemas, worker
lifecycle, shared-memory transport, or semantic routing.
