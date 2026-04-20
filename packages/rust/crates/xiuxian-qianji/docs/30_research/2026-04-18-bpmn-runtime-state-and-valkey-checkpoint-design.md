---
type: knowledge
title: "Design Note: qianji-bpmn-engine Runtime State and Checkpoint Storage Model"
category: "research"
status: "draft"
authors:
  - codex
created: 2026-04-18
tags:
  - qianji
  - bpmn
  - valkey
  - checkpoint
  - runtime
  - design
---

# Design Note: qianji-bpmn-engine Runtime State and Checkpoint Storage Model

## 1. Purpose

This note narrows the planning lane opened in
[Research Plan: qianji-bpmn-engine Architecture and xiuxian-qianji Integration](2026-04-18-bpmn-engine-research-plan.md)
to one concrete question:

How should `qianji-bpmn-engine` represent runtime state and persist checkpoint
state without compromising hot-path performance, while keeping Valkey as the
distributed default and leaving room for one lightweight local SQL option?

This is still a planning artifact. It does not freeze the final Rust API, but
it does fix the intended implementation shape strongly enough for architecture
audit.

## 2. Working Constraints

The current design constraints are:

1. `qianji-bpmn-engine` is a standalone crate.
2. `xiuxian-qianji` depends on it through thin host adapters.
3. BPMN runtime semantics stay inside `qianji-bpmn-engine`.
4. Valkey remains the distributed/default checkpoint path for v1.
5. One feature-gated local SQL path is acceptable for lightweight client-side
   persistence when distributed writer ownership is not required.
6. The runtime hot path must avoid XML walking, repeated string-heavy lookup,
   and checkpoint writes on every internal transition.
7. Optimize state shape and write cadence before spending effort on exotic
   checkpoint codecs.

## 3. Runtime State Split

The implementation should use a strict split between immutable parsed specs and
mutable per-instance state.

### 3.1 Immutable Spec Layer

The parsed process/package spec should be cold shared data:

```rust
struct ProcessKey {
    package_id: Arc<str>,
    process_id: Arc<str>,
    spec_digest_hex: Arc<str>,
}

struct BpmnProcessSpec {
    key: ProcessKey,
    nodes: Vec<NodeSpec>,
    edges: Vec<EdgeSpec>,
    incoming_offsets: Vec<std::ops::Range<u32>>,
    outgoing_offsets: Vec<std::ops::Range<u32>>,
    boundary_attachments: Vec<BoundaryAttachment>,
    join_kinds: Vec<JoinKind>,
    event_templates: Vec<EventSubscriptionTemplate>,
}
```

The important property is not the exact field naming. The important property is
that:

1. the spec is immutable after parse/build
2. ids are normalized into compact indices
3. graph lookup tables are precomputed
4. the spec can be wrapped in `Arc` and shared across many workflow instances

### 3.2 Mutable Instance Layer

Each running workflow instance should keep only compact mutable execution
state:

```rust
struct BpmnInstanceState {
    instance_id: Arc<str>,
    process: ProcessKey,
    sequence: u64,
    lifecycle: InstanceLifecycle,
    variables: serde_json::Value,
    node_states: Vec<NodeRuntimeState>,
    active_tokens: Vec<TokenRecord>,
    joins: Vec<JoinRuntimeState>,
    waits: Vec<WaitRegistration>,
    pending_host_work: Option<PendingHostWork>,
    suspend_reason: Option<SuspendReason>,
    updated_at_ms: u64,
}
```

Key design intent:

1. `variables` may remain flexible JSON because the host boundary already uses
   JSON-shaped workflow context in Qianji today.
2. `node_states`, `active_tokens`, `joins`, and `waits` should be index-based
   dense collections rather than maps keyed by BPMN ids.
3. `pending_host_work` stores only recoverable coordination data such as a host
   work id, request kind, and target node index. It must not store full host
   objects or executor handles.

## 4. Hot Path Execution Model

The hot path should follow this loop:

1. resolve the current token frontier by numeric node index
2. consult precomputed spec tables for outgoing edges, boundary attachments,
   event subscriptions, and join semantics
3. mutate in-memory instance state
4. only cross the host boundary when a BPMN node needs external work or enters
   an externally visible waiting state

That means the engine should spend most transitions on:

1. `Vec` indexing
2. compact enum branching
3. bounded JSON variable merges
4. local token/join/wait bookkeeping

It should not spend most transitions on:

1. reparsing XML
2. scanning hash maps by BPMN string ids
3. reconstructing execution state from serialized blobs
4. synchronous checkpoint writes for every internal gateway hop

## 5. Valkey Key Schema

The v1 checkpoint layout should stay simple.

### 5.1 Required Keys

1. `xq:bpmn:ckpt:<instance_id>:state`
   Stores the full checkpoint payload for the latest durable instance state.
2. `xq:bpmn:ckpt:<instance_id>:lease`
   Optional host-side writer-ownership key when distributed execution or remote
   possession must prevent stale writers.

### 5.2 Payload Shape

The state key should embed operational metadata directly in the payload instead
of splitting v1 into many auxiliary keys.

Illustrative payload shape:

```json
{
  "version": 1,
  "sequence": 42,
  "instance_id": "wf_123",
  "process": {
    "package_id": "pkg",
    "process_id": "approve_invoice",
    "spec_digest_hex": "..."
  },
  "lifecycle": "waiting_external_event",
  "variables": {},
  "node_states": [],
  "active_tokens": [],
  "joins": [],
  "waits": [],
  "pending_host_work": null,
  "suspend_reason": null,
  "updated_at_ms": 1760000000000
}
```

### 5.3 Why One Main State Key

The v1 preference for one main state key is deliberate:

1. simpler atomic updates
2. simpler resume semantics
3. fewer Valkey round-trips
4. easier debugability during early adoption

Secondary index keys are explicitly deferred until a real query or coordination
need justifies them.

## 5.4 Local SQL Client Store

The local SQL path should stay intentionally narrow.

1. it is a feature-gated lightweight client option, not the distributed writer
   path
2. it should store the same JSON checkpoint envelope shape as the Valkey path
3. it should keep the same monotonic sequence guard so stale local saves are
   rejected deterministically
4. it does not need lease ownership because the bounded target is local
   client-side persistence rather than multi-writer orchestration

## 6. Checkpoint Codec Choice

The v1 checkpoint payload should use JSON.

This is not because JSON is the fastest possible codec. It is because:

1. current `xiuxian-qianji` checkpointing already uses JSON in Valkey
2. checkpoint writes are intentionally pushed off the hottest internal routing
   path
3. debugability and operational inspection matter during the early rollout
4. state shape and write cadence are higher-leverage performance wins than
   swapping codecs prematurely

Deferred optimization rule:

Only revisit a binary codec if measurements show checkpoint serialization itself
is a material bottleneck on suspend/resume or external-wait transitions.

## 7. Write Policy

The write policy should prioritize recoverability at semantic boundaries, not at
every internal token movement.

### 7.1 Must-Save Transitions

Persist immediately when the instance:

1. enters a suspend state
2. enters `waiting_external_event`
3. enters `waiting_user_action`
4. hands off external work and needs durable recovery of that handoff
5. completes successfully
6. fails terminally

### 7.2 Bounded Background Safety Save

For long-running internal transitions that do not hit a blocking boundary, use a
bounded dirty-state flush policy rather than saving on every hop.

Working default for audit:

1. maintain an in-memory dirty flag
2. track transition count since the last durable save
3. allow a periodic flush threshold such as "every 64 transitions" or a short
   elapsed-time threshold when the instance remains runnable for an extended
   period

The exact thresholds are implementation details to benchmark, but the principle
is fixed: no full checkpoint write per token move.

### 7.3 Atomicity Rule

If the host guarantees a single logical writer for one workflow instance, a
single `SET ... EX ...` style update of the state key is acceptable.

If stale writers are possible, use one of:

1. lease key with TTL plus writer identity
2. sequence-protected atomic write via pipeline or Lua compare-and-set

The sequence field in the checkpoint payload exists precisely to support this
upgrade path.

## 8. TTL and Cleanup

The working default should match the current Qianji checkpoint posture:

1. apply TTL to abandoned checkpoints
2. explicitly delete checkpoints on clean completion

## 9. Status Update After Checkpoint Lease Ownership Slice

The first bounded checkpoint slice has now landed in the BPMN crate.

Current implemented status:

1. `BpmnCheckpointEnvelope` remains the durable state payload
2. checkpoint codec remains JSON
3. `save_checkpoint` now writes the envelope to
   `xq:bpmn:ckpt:<instance_id>:state`
4. `load_checkpoint` now reads and decodes the same state key
5. the initial TTL matches the current Qianji checkpoint posture at seven days
6. stale-writer rejection now uses a sequence-protected atomic Lua compare-and-set
   on the state key
7. BPMN checkpoint leases now support bounded acquire, renew, and release on
   `xq:bpmn:ckpt:<instance_id>:lease`
8. `save_checkpoint_as_owner(...)` now requires lease ownership before writing
   the state key
9. lease renewal remains explicit; the BPMN crate does not yet spawn automatic
   background renew workers
10. do not depend on TTL as the normal cleanup path

For early implementation planning, keeping the current 7-day state-key TTL
baseline is a reasonable compatibility choice unless BPMN-specific retention
pressure proves otherwise.

## 9. What Stays in xiuxian-qianji

`xiuxian-qianji` should own only the thin host-facing surfaces:

1. resolving the effective Valkey URL
2. implementing the `BpmnHostBridge` trait
3. mapping BPMN service/user/manual tasks to existing Qianji executors or
   orchestration entrypoints
4. publishing runtime telemetry through existing Valkey-backed emitters or
   higher-level sinks
5. exposing CLI/app/user-facing surfaces

`xiuxian-qianji` should not own:

1. BPMN token semantics
2. BPMN join logic
3. BPMN wait-registration semantics
4. BPMN checkpoint serializer versioning

## 10. Non-Goals for V1

1. broad multi-backend checkpoint abstraction
2. binary checkpoint codec by default
3. storing parsed BPMN XML in checkpoint storage
4. splitting checkpoints into many coordination keys before measured need
5. embedding host executor internals inside checkpoint payloads

## 13. Status Update After DB Store SQL Checkpoint Slice

The crate now supports one bounded lightweight client-side checkpoint option in
addition to the existing Valkey path.

Current implemented status:

1. the workspace storage facade is now `xiuxian-db-store`
2. the facade keeps the existing heavy `vector-store` surface feature-gated
3. the facade now also exposes one `sqlite` feature backed by SQLite helpers
4. `qianji-bpmn-engine` now keeps Valkey as the distributed/default checkpoint
   path
5. `qianji-bpmn-engine` additionally exposes feature-gated local SQL
   checkpoint save/load functions for lightweight client-side persistence
6. the SQL path keeps the same monotonic sequence-guard behavior as the Valkey
   path, but intentionally does not introduce lease ownership

## 11. Audit Summary

The implementation direction is now concrete enough to audit:

1. parse once into immutable indexed specs
2. run on compact mutable instance state
3. keep runtime process resolution on cached indices instead of repeated
   process-id scans
4. build adjacency indexes as dense arrays rather than temporary nested edge
   buckets

## 12. Status Update After Performance Hot-Path Slice

The first explicit performance cleanup slice has now landed on top of the
bounded runtime and checkpoint work.

Current implemented status:

1. `BpmnInstanceState` now stores a cached `process_index` alongside the
   existing `ProcessKey`
2. runtime and wait paths validate that cached index against `process_id` and
   repair stale values when recovering older checkpoints
3. checkpoint JSON decode remains backward-compatible when the serialized state
   has no `process_index` field
4. `BpmnProcessSpec::new(...)` now builds incoming/outgoing adjacency tables
   through a two-pass dense writer instead of temporary `Vec<Vec<u32>>`
   buckets
5. ignored local perf probes are now checked in so the crate can emit timing
   evidence without making default CI timing-sensitive

Local probe evidence from this worktree:

1. process lookup over 20,000 processes and 200,000 iterations:
   `linear_ms=49192.000`, `indexed_ms=2.576`
2. adjacency-index construction over a 10,000-node / 9,999-edge linear graph
   for 100 iterations:
   `legacy_ms=217.560`, `dense_ms=78.885`

These numbers are local probe evidence rather than portable benchmark gates,
but they are sufficient to prove the direction of the optimization: remove
repeated string-key scans and temporary bucket allocation from the BPMN hot
path before expanding semantics.
3. store only recovery-relevant runtime state in Valkey
4. use JSON payloads for v1 because checkpoint writes are off the hottest path
5. save at semantic recovery boundaries, not on every token move
