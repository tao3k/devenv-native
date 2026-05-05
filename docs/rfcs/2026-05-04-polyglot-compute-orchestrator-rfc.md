---
type: knowledge
title: "RFC: Polyglot Compute Orchestrator (Rust/Python/Julia)"
category: "rfc"
status: "draft"
authors:
  - codex
created: 2026-05-04
tags:
  - rfc
  - orchestrator
  - wendao
  - polyglot
  - python
  - julia
  - docling
  - arrow-flight
metadata:
  title: "RFC: Polyglot Compute Orchestrator (Rust/Python/Julia)"
---

# RFC: Polyglot Compute Orchestrator (Rust/Python/Julia)

## 1. Summary

This RFC proposes a bounded polyglot compute coordination lane inside the
existing Wendao runtime, document-extraction provider, and Julia plugin
contracts. A future dedicated Rust crate, temporarily named
`xiuxian-polyglot-orchestrator` or `xiuxian-dispatch`, remains a deferred
package-boundary option rather than the first implementation step.

The core decision is to formalize Rust as the admission-control, lifecycle,
fallback, and validation plane that bridges two different execution
environments without creating a second document-extraction service, a second
Julia scheduler, or a second semantic authority:

1. **The Python Lane (Docling)**: Heavy, memory-intensive, process-bound
   document parsing and vision modeling exposed through the existing Wendao
   document-extraction Flight service and OCR shard contract.
2. **The Julia Lane (Compute)**: Multi-threaded, JIT-compiled scientific and
   relational compute exposed through existing `xiuxian-wendao-julia`
   family/profile contracts.

To achieve this, the coordination lane will enforce a strict separation of
concerns:

- **Control Plane**: Rust-owned runtime policy, health, admission control,
  backpressure, and status metadata. A dedicated gRPC API is optional future
  transport, not a first-slice requirement.
- **Data Plane**: Apache Arrow Flight and Arrow IPC-compatible batches as typed
  transport contracts. Cross-process zero-copy requires a separate shared-memory
  pilot with explicit allocator, descriptor, lifetime, and cleanup semantics.

## 2. Problem Statement

Xiuxian Artisan Workshop relies on heterogeneous compute models. Python
excels at deep learning models and document parsing (e.g., Docling). Julia
excels at high-concurrency scientific compute and numerical reranking.

However, bridging these environments introduces severe systemic risks:

1. **The Python OOM/GIL Trap**: Python workers processing large PDFs or images
   can consume unpredictable amounts of memory. Without a system-level
   scheduler, concurrent requests will either block on the GIL or OOM the host.
2. **The Julia JIT Storm**: Julia requires warmup. A burst of concurrent
   requests to a cold Julia worker pool leads to timeout cascading.
3. **The Serialization Tax**: Passing gigabytes of extracted document tables
   and vectors from Python to Rust, and then to Julia via JSON or MsgPack,
   destroys performance.

We need a Rust-owned coordination layer capable of managing worker lifecycles,
memory pressure, request budgets, fallback behavior, and typed Arrow routing
without bypassing the existing provider boundaries.

## 3. Architecture: The Triple-Dispatch Pattern

The coordination lane will implement a bounded "Triple-Dispatch Pattern":

### 3.1 The Rust Control Plane (Master Scheduler)

Rust acts as the central admission and validation layer using `tokio`'s
asynchronous primitives.

- **Worker Bindings**: Reuses existing Flight endpoints, provider bindings,
  route metadata, and health probes for Python and Julia worker services.
- **Backpressure & Wave Dispatch**: Uses semaphores to cap the number of active
  Python Docling processes. If the Python pool is saturated, Rust queues or
  gracefully rejects tasks based on downstream capacity.
- **Julia Admission Control**: Gates high-concurrency numeric requests by
  capability family, profile route, timeout, and `max_in_flight_requests`.
  Julia owns internal `Threads.@spawn` scheduling unless a future worker-queue
  protocol exposes explicit Julia-side queue ownership.

### 3.2 The Data Plane (Arrow Flight, IPC, and Future Shared Memory)

Large data should not traverse a control channel.

1. Python (`docling`) parses a document and materializes the output directly
   into the existing document resource, structure, or OCR shard Arrow schemas.
2. Python exposes those rows through the existing document extraction Flight
   service or through approved Arrow IPC artifacts owned by the Rust provider.
3. Rust validates schema version, source identity, shard identity, ordering,
   provenance, and failure rows before forwarding any derived compute request.
4. Julia receives only bounded, versioned Arrow-shaped projections and returns
   advisory compute rows to Rust.

This RFC does not claim cross-process zero-copy as a landed contract.
`__arrow_c_array__` and related PyCapsule methods are in-process export
interfaces. Arrow Flight transports Arrow IPC frames. A true shared-memory
pilot must define, at minimum, the allocator (`mmap`, `memfd`, or equivalent),
descriptor format, process ownership, cleanup semantics, crash recovery,
container boundary behavior, and replay validation.

**Transport Configuration**: HTTP/2's default initial flow-control window is
65,535 octets, while the default max frame size is 16,384 octets. Tuning should
target measured Flight data paths, Tonic message-size ceilings, stream and
connection windows, and route-specific batch sizing. A blanket 4 MiB
control-plane window is not a first-slice requirement.

### 3.3 M:N Concurrency Mapping

- **Rust -> Python**: M tasks mapped to N isolated processes (to bypass the GIL).
- **Rust -> Julia**: M tasks mapped to N Julia OS threads (`Threads.@spawn`).
  Rust controls the admission rate and may pass deployment-level affinity
  policy hints. Julia owns the actual task scheduling and thread execution
  inside the worker process.

## 4. The Python Lane (Docling Integration)

The Python lane must extend the existing Wendao analyzer service rather than
introducing a parallel Docling wrapper.

- **Existing service**: `DocumentExtractFlightServer` already exposes
  `/analysis/document-extract` and `/analysis/pdf-ocr-shards`.
- **Existing contracts**: Rust already owns document extraction modes, OCR shard
  input/result schemas, worker-budget metadata, cache and ordering validation,
  and the stable document resource table.
- **Model reality**: Docling's current model catalog includes Heron as the
  default layout model, but model speed and memory behavior must be treated as
  deployment-specific evidence rather than RFC-level guarantees.
- **Lifecycle**: Rust should continue to send bounded worker budgets to Python
  and collect status/latency/error evidence. Process kill/respawn policy should
  be introduced only after a provider-owned worker supervisor contract is
  defined and validated.

## 5. The Julia Lane (Compute Integration)

Julia acts as the high-concurrency reduction and numerical lane.

### 5.1 Schema-Aware Warmup (JIT Pre-compilation)

Even with a persistent gRPC server, Julia's JIT compiler triggers on the first
call of a function with a **specific type signature**. To eliminate
"Time To First Batch" (TTFB) latency:

- **Warmup Protocol**: Rust may send an initial empty or tiny Arrow batch that
  matches the selected profile schema.
- **Specialization**: This encourages Julia to compile the Arrow-to-Struct
  projections and the specific numerical kernels (e.g., Rerank, Scoring)
  before real traffic arrives.
- **Readiness Gate**: The orchestrator will not promote the Julia worker to
  `ACTIVE` status until schema validation, route validation, health checks, and
  locally benchmarked warmup thresholds pass. A fixed 5 ms threshold is not a
  first-slice invariant.

### 5.2 HPC Thread Management (`ThreadPinning.jl`)

To maximize L1/L3 cache efficiency and memory bandwidth:

- **Affinity Policy**: Julia workers utilize `ThreadPinning.jl` to pin compute
  threads to physical cores (e.g., `pinthreads(:cores)`).
- **OS-led Optimization**: Deployment may provide affinity hints through
  process launch policy, environment, cgroups, or taskset. Rust must not assume
  affinity is available on every platform.
- **Interference Mitigation**: The first slice should record topology and
  thread-pinning diagnostics as evidence before enforcing socket or NUMA
  placement policy.

### 5.3 Execution and Data Flow

- **Execution**: Julia receives a Flight ticket, pulls the Arrow data, and
  executes bounded multi-threaded operations utilizing `OhMyThreads.jl` for
  efficient, lock-free parallel mapping.
- **Result**: Julia returns the computed outputs back to Rust via Flight `DoPut`
  or as a direct `RecordBatch` response.

## 6. Physical Layout (Proposed)

No new crate is approved in the first slice. The first implementation should
extend existing owners:

1. `xiuxian-wendao-runtime` for reusable Flight client configuration,
   route-level request gates, timeout policy, and transport validation.
2. `xiuxian-wendao` and `xiuxian-wendao-attachments` for document extraction
   provider policy, OCR shard scheduling, ordering validation, and cache
   ownership.
3. `xiuxian-wendao-julia` for Julia-specific capability/profile contracts,
   request/response schemas, route validation, and advisory evidence decoding.
4. `xiuxian-wendao-analyzer` for the Python Docling Flight worker surface.

A future crate may be created only if these owners accumulate duplicated,
generic coordination code that cannot remain local without obscuring
boundaries. If that trigger fires, the deferred layout would be:

```text
src/
  lib.rs
  pool/
    python_worker.rs   # Process lifecycle, OOM monitoring, pessimistic quotas
    julia_worker.rs    # Thread-group management, JIT heartbeat & readiness probes
  transport/
    flight_bus.rs      # Arrow Flight IPC/TCP routing (shm integration)
    control_grpc.rs    # Optional command signaling and measured transport tuning
  scheduler/
    backpressure.rs    # Tokio semaphores and admission control
    strategy.rs        # Load-balancing algorithms
```

## 7. Alignment with Existing Architecture

This RFC strictly adheres to the principles established in:

- **Arrow Schema-First Julia Compute ABI**
  (`2026-04-06-arrow-schema-first-julia-compute-abi-for-wendao-memory-family.md`):
  Julia remains a compute-only plugin; Rust retains state authority.
- **Python Arrow Flight Boundary and Removal Program**
  (`2026-03-29-python-arrow-flight-boundary-rfc.md`): Python remains a narrow
  transport and model-worker surface, not a local orchestration center.
- **Wendao document extraction and attachment boundaries**: Docling remains the
  document/OCR authority until a later approved benchmark gate accepts a faster
  or hybrid path.

The Repo-Native Semantic SSOT RFC remains a possible future consumer of this
compute evidence. It is not a dependency for this compute orchestration lane
until the semantic SSOT layer is approved and physically initialized.

## 8. Data Schema & Heterogeneous Tables

Bridging Docling's layout-aware parsing into Julia compute requires schema
stabilization, but the strategy must be selected from evidence.

### 8.1 Candidate Schema Strategies

The first pilot should benchmark at least these shapes:

1. **Profile-specific schemas**: one stable Arrow schema per compute profile.
   This is the current Julia memory-compute ABI pattern and should be the
   default starting point.
2. **Normalized long tables**: represent heterogeneous Docling fields as
   `document_id`, `block_id`, `field_name`, `field_value`, type, confidence,
   and provenance rows.
3. **Nested or struct-heavy schemas**: preserve hierarchy and provenance while
   bounding top-level column count.
4. **Global super-schema**: a wide sparse schema that is allowed only as a
   benchmark candidate, not as a mandated design.

Polars and Arrow PyCapsule may help within Python-local conversion or
in-process interop, but they do not define the cross-process transport contract
by themselves.

### 8.2 Benchmark Requirements

Any schema strategy must report:

1. Arrow batch size and metadata size
2. Julia compile/warmup time by profile
3. steady-state compute time
4. null density and active-column distribution
5. cache and memory pressure
6. schema evolution cost
7. validation and replay behavior

SIMD or validity-bitmap optimizations may be recorded as implementation
evidence only after local benchmarks demonstrate the effect for the selected
Arrow and Julia stack.

## 9. Rollout Plan

1. **Phase 1 (Boundary Calibration)**: Extend the RFC, audit, and tracking
   artifacts so the lane reuses existing Wendao runtime, document extraction,
   attachments, analyzer, and Julia plugin boundaries.
2. **Phase 2 (Python Pressure Control)**: Add or refine Rust-side evidence for
   document extraction worker budgets, queue pressure, failure rows, and OCR
   shard ordering without introducing a new Python wrapper.
3. **Phase 3 (Julia Readiness)**: Add profile-level health, warmup, and
   readiness evidence on top of existing `memory.julia_compute` and graph
   structural contracts.
4. **Phase 4 (Schema Benchmark)**: Compare profile-specific, normalized,
   nested, and super-schema strategies before approving any heterogeneous-table
   default.
5. **Phase 5 (Shared Memory Pilot, Optional)**: Define and benchmark an
   explicit mmap/memfd/UDS shared-memory contract only if Flight/IPC evidence
   shows transport copying is the dominant bottleneck.
