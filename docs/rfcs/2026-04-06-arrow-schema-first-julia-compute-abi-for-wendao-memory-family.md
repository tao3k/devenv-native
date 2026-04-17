---
type: knowledge
title: "RFC: Arrow Schema-First Julia Compute ABI for the Wendao Memory Family"
category: "rfc"
status: "draft"
authors:
  - "Wendao / Xiuxian maintainers"
created: 2026-04-06
tags:
  - rfc
  - wendao
  - memory
  - julia
  - arrow
  - abi
  - flight
metadata:
  title: "RFC: Arrow Schema-First Julia Compute ABI for the Wendao Memory Family"
---

# RFC: Arrow Schema-First Julia Compute ABI for the Wendao Memory Family

## Status

Draft

## Authors

- Wendao / Xiuxian maintainers

## Date

2026-04-06

## 1. Abstract

This RFC defines an **Arrow Schema-First Julia Compute ABI** for the Wendao
`memory` family.

The governing principle is:

> **Julia owns high-performance scientific and mathematical compute only. Rust
> remains the authoritative owner of host state, lifecycle, registry,
> fallback, audit, and final mutation decisions.**

This RFC extends the existing scorer-style Arrow contract into a first
normative family-level ABI for memory-family compute. It does **not** transfer
memory authority to Julia. It does **not** introduce a separate
`PluginKit.jl`. Instead, it expands `WendaoArrow.jl` into the Julia-side
**transport + contract + authoring substrate for compute plugins only**.

The existing scorer-style Arrow contract remains valid as the base precedent
and is generalized into **family/profile** form rather than replaced from
scratch.

## 2. Motivation

Wendao already has a stable Rust-side memory substrate.
`xiuxian-memory-engine` is currently a Rust-based self-evolving memory engine
with implemented episode storage, Q/utility learning, two-phase recall,
lifecycle control, and read-only memory projections.

At the same time, Wendao already has a working Julia transport/contract lane:

- `WendaoArrow.jl` owns the Julia-side Arrow/Flight transport and contract
  layer and is Flight-only
- `xiuxian-wendao-julia` already acts as an ecosystem-thin Julia bridge over a
  runtime-owned generic Flight seam
- `WendaoAnalyzer.jl` has already demonstrated the intended split:
  transport/contract in `WendaoArrow.jl`, domain compute in Julia, and request
  shaping, fallback, and validation in Rust

The remaining gap is not transport. The remaining gap is a **compute-only ABI**
that lets Julia provide high-performance memory-family computation without
inheriting host authority.

## 3. Non-Goals

This RFC does **not**:

- move authoritative memory ownership to Julia
- expose `EpisodeStore` internals directly to Julia
- let Julia mutate host memory state directly
- let Julia own lifecycle transitions
- let Julia directly govern durable knowledge promotion
- define a full all-family plugin ecosystem rewrite
- introduce a standalone `PluginKit.jl`
- scaffold `.data/WendaoMemory.jl` in this RFC
- remove the current scorer-style Arrow ABI

## 4. Governing Principle

The following principle is normative:

> **Julia compute services are compute-only. Rust host authority is
> non-transferable.**

Concretely:

- Julia may score, rerank, calibrate, optimize, estimate uncertainty, and
  solve bounded scientific or mathematical subproblems.
- Rust host retains final authority over:
  - state ownership
  - lifecycle transitions
  - registry writes
  - audit events
  - fallback/degrade policy
  - final mutation decisions

## 5. Existing Boundary Alignment

This RFC aligns with current system boundaries:

- `WendaoArrow.jl` already owns Julia-side transport and contract, not
  analyzer/domain policy
- `xiuxian-wendao-julia` already owns Julia-specific manifest discovery, route
  defaults, typed validation/decoding, and minimal binding glue over a
  runtime-owned generic Flight seam
- `xiuxian-memory-engine` remains the authoritative Rust memory engine, not a
  Julia-owned subsystem
- the cross-layer memory boundary is already governed by
  [RFC: Wendao Memory Layer Boundaries](./2026-04-05-wendao-memory-layer-boundaries-rfc.md)
- the memory architecture note already points at this RFC as the compute-only
  ABI boundary in
  [architecture.md](../01_core/memory/architecture.md)

## 6. High-Level Architecture

### 6.1 Rust Host

Rust continues to own:

- authoritative memory state
- lifecycle transitions
- state backend
- fallback/local implementation
- audit/event emission
- final mutation decisions

### 6.2 `xiuxian-wendao-runtime`

Runtime continues to own:

- generic Flight client
- transport negotiation
- timeout policy
- route normalization
- batch execution

### 6.3 `xiuxian-wendao-julia`

The Julia bridge remains ecosystem-thin and owns only:

- Julia-specific capability-manifest discovery
- Julia route defaults
- typed memory-family row validation/decoding
- minimal Julia binding glue
- plugin-owned host-adapter helpers over Rust read-only projections and
  evidence rows

### 6.4 `WendaoArrow.jl`

`WendaoArrow.jl` becomes the Julia-side:

- transport
- contract
- authoring substrate

for **compute plugins only**.

It MUST NOT own:

- lifecycle
- state mutation
- registry authority
- host policy

## 7. Contract Model

This RFC defines a three-layer ABI.

### 7.1 Physical Arrow Schema

Defines:

- column names
- Arrow types
- nullability
- additive-column rules
- batch/stream semantics

This stays aligned with the existing Arrow schema-first contract model already
used by WendaoArrow.

### 7.2 Semantic Contract

Defines:

- join keys
- row-order assumptions
- duplicate handling
- score/verdict meaning
- fallback triggers
- required response semantics

### 7.3 Capability Contract

Defines:

- `family`
- `capability_id`
- `profile_id`
- `request_schema_id`
- `response_schema_id`
- `route`
- `schema_version`
- `health_route`
- `timeout_secs`
- `scenario_pack`

## 8. First Standardized Family: `memory`

The first ABI family standardized by this RFC is one umbrella `memory`
family.

### Profiles

- `episodic_recall`
- `memory_gate_score`
- `memory_plan_tuning`
- `memory_calibration`

These are **profiles under one family**, not four unrelated families.

## 9. Ownership and Mutation Invariants

The following invariants are normative:

1. **Julia compute services MUST NOT mutate authoritative host memory state.**
2. **Julia compute services MUST NOT own lifecycle transitions.**
3. **Julia compute services MUST NOT directly govern durable knowledge
   promotion.**
4. **Rust host MUST materialize read-only projection batches before Julia
   compute is invoked.**
5. **Any Julia-side result that implies a state change MUST be treated as
   recommendation-only until Rust host commits it.**
6. **Julia compute services MUST NOT introduce hidden host mutation channels
   via metadata, side-channel flags, or implicit response conventions.**

## 10. Allowed Julia Compute Scope

Julia compute services MAY own:

- scientific scoring kernels
- reranking kernels
- uncertainty estimation
- calibration
- threshold fitting
- optimization
- scenario-specific scientific models
- solver-backed bounded compute
- mathematical/statistical recommendation generation

Julia compute services MUST NOT own:

- state storage
- state mutation
- lifecycle advancement
- registry authority
- durable knowledge authority
- host fallback control
- authoritative audit emission

## 11. Host Read Model

Julia never consumes `EpisodeStore` internals directly.

Instead:

1. Rust host materializes a canonical **read-only projection batch** or
   snapshot.
2. That projection batch is encoded as Arrow with the appropriate schema and
   metadata.
3. Julia compute depends only on:
   - Arrow schema
   - schema metadata
   - capability manifest
   - family/profile contract

This keeps Julia decoupled from Rust-internal memory-store implementation
details.

## 12. Minimal Compat Principle

To keep `xiuxian-wendao-julia` as thin as possible, the following rules are
normative:

- Rust host MUST NOT grow one bespoke adapter per Julia package.
- Rust host SHOULD implement **family-level adapters** only.
- New Julia packages inside an existing family/profile SHOULD require
  manifest/schema additions only, not new host business adapters.
- `xiuxian-wendao-julia` MUST remain ecosystem-thin and MUST NOT grow a second
  host-local business adapter layer.

## 13. Public Julia Authoring Surface in `WendaoArrow.jl`

`WendaoArrow.jl` SHOULD expose the following normative authoring surfaces:

- `WendaoArrow.Capability`
- `WendaoArrow.Manifest`
- `WendaoArrow.Contracts`
- `WendaoArrow.Services`
- `WendaoArrow.Validate`

These surfaces are authoring helpers for compute plugins only. They do not
carry host policy or lifecycle ownership.

## 14. Capability Manifest Contract

A memory-family capability manifest entry MUST carry:

- `family = "memory"`
- `capability_id`
- `profile_id`
- `request_schema_id`
- `response_schema_id`
- `route`
- `schema_version`
- `enabled`

It MAY also carry:

- `health_route`
- `timeout_secs`
- `max_in_flight_requests`
- `scenario_pack`

## 15. Runtime Config Contract

The normative host config surface is runtime-level, not repo-plugin-level.

```toml
[memory.julia_compute]
enabled = true
base_url = "grpc://127.0.0.1:18825"
schema_version = "v1"
timeout_secs = 3
max_in_flight_requests = 32
fallback_mode = "rust"
shadow_compare = true

[memory.julia_compute.routes]
episodic_recall = "/memory/episodic_recall"
memory_gate_score = "/memory/gate_score"
memory_plan_tuning = "/memory/plan_tuning"
memory_calibration = "/memory/calibration"
```

Required fields:

- `enabled`
- `base_url`
- `schema_version`
- `timeout_secs`
- `max_in_flight_requests`
- `fallback_mode`
- `shadow_compare`
- route mapping for the four memory profiles

Default rollout posture:

- `fallback_mode = "rust"`
- `shadow_compare = true`
- `max_in_flight_requests = 32`

## 16. Profile Intent

### 16.1 `episodic_recall`

This is a **retrieval profile**, not a scorer profile.

It operates over a Rust-owned read-only memory projection. Julia MAY perform:

- retrieval compute
- reranking
- uncertainty estimation
- scenario-aware scoring

Julia MUST NOT perform:

- candidate persistence
- lifecycle mutation
- state writes

### 16.2 `memory_gate_score`

This profile is **recommendation-only**.

It MAY return:

- scoring
- confidence
- suggested verdict, including working-knowledge promotion recommendations only
- suggested next action

It MUST NOT trigger state transition by itself.

### 16.3 `memory_plan_tuning`

This profile is **advice-only**.

It MAY return:

- parameter recommendations
- budget recommendations
- tuning diagnostics

It MUST NOT mutate active host config directly.

### 16.4 `memory_calibration`

This profile is **artifact/recommendation-only**.

It MAY return:

- thresholds
- weights
- calibration artifacts
- metric summaries

It MUST NOT auto-activate calibrated outputs.

## 17. Canonical Schema Fragments

To reduce schema duplication, the RFC standardizes reusable fragments.

### 17.1 `identity_fragment`

Example fields:

- `row_id`
- `candidate_id`
- `scope`

### 17.2 `score_fragment`

Example fields:

- `semantic_score`
- `utility_score`
- `final_score`
- `confidence`

### 17.3 `verdict_fragment`

Example fields:

- `verdict`
- `reason`
- `next_action`

### 17.4 `tuning_fragment`

Example fields:

- `k1`
- `k2`
- `lambda`
- `min_score`
- `max_context_chars`

### 17.5 `memory_projection_fragment`

This fragment is required for read-only Rust → Julia memory projections.

Example fields:

- `intent_embedding`
- `q_value`
- `success_count`
- `failure_count`
- `retrieval_count`
- `created_at_ms`
- `updated_at_ms`
- `scope`

Profile schemas SHOULD be composed from these fragments plus profile-specific
additive columns.

## 18. Compatibility with Existing Scorer Contract

The current scorer-style Arrow contract remains valid.

This RFC does not discard it. Instead, it treats it as the base ABI precedent
and generalizes it into **family/profile** form. Existing scorer-style
capabilities remain representable under the new model.

## 19. Errors, Fallback, and Shadow Compare

### 19.1 Fallback

Schema mismatch, invalid rows, timeout, transport failure, or response
validation failure MUST trigger Rust fallback.

### 19.2 Shadow Compare

When `shadow_compare = true`, the host SHOULD record at least:

- recall rank drift
- gate verdict drift
- confidence drift
- timeout/fallback rate
- schema validation failure rate

Shadow compare MUST NOT change authoritative host decisions by itself.

## 20. Documentation Sync

This RFC is the primary deliverable for the current documentation slice.

After landing:

- keep a short pointer in [architecture.md](../01_core/memory/architecture.md)
  to the compute-only ABI boundary
- keep the WendaoArrow contract docs aligned so the scorer contract points to
  this RFC as the memory-family extension
- keep terminology aligned across:
  - this RFC
  - the memory architecture note
  - the WendaoArrow contract docs

## 21. Deferred Work

This RFC intentionally defers:

- `.data/WendaoMemory.jl` package scaffolding
- Rust execution seams beyond minimal config/validation integration
- non-memory families
- wider ecosystem ABI generalization
- direct package structure assumptions for `WendaoMemory.jl`

## 22. Acceptance Criteria

The RFC is accepted when:

- compute-only ownership is explicit in title, summary, architecture, and
  invariants
- `memory_gate_score`, `memory_plan_tuning`, and `memory_calibration` are
  recommendation-only
- the RFC defines a read-only host projection model
- the RFC introduces the minimal compat principle
- the RFC introduces canonical schema fragments, including
  `memory_projection_fragment`
- default rollout posture remains `fallback_mode = "rust"` and
  `shadow_compare = true`
- terminology stays aligned across the RFC, memory architecture note, and
  WendaoArrow docs
- repository checks remain clean, including `git diff --check`

## 23. Assumptions

- `.data/WendaoMemory.jl` currently has no scaffold
- the first ABI generalization is memory-family-first, not full
  ecosystem-wide
- `WendaoArrow.jl` absorbs the authoring substrate
- no standalone `PluginKit.jl` will be proposed in this RFC
