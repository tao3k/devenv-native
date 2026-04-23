---
type: knowledge
title: "Design Note: qianji-bpmn-engine Crate Skeleton and Host Bridge"
category: "research"
status: "draft"
authors:
  - codex
created: 2026-04-18
tags:
  - qianji
  - bpmn
  - crate
  - api
  - host-bridge
  - design
---

# Design Note: qianji-bpmn-engine Crate Skeleton and Host Bridge

## 1. Purpose

This note fixes the intended crate boundary for `qianji-bpmn-engine` before any
Rust code is written.

It answers four implementation-shaping questions:

1. what the first crate layout should be
2. what the minimal public API should expose
3. what the host bridge trait should own
4. what must remain outside the crate in `xiuxian-qianji`

This is still a planning note. It is not a commitment to exact symbol names,
but it is intended to prevent architectural drift during the first scaffold
slice.

## 2. Crate Layout

The first scaffold should favor explicit feature folders over flat files.

Target layout:

```text
packages/rust/crates/qianji-bpmn-engine/
  Cargo.toml
  src/
    lib.rs
    error.rs
    parser/
      mod.rs
      package.rs
      validate.rs
      import.rs
      normalize.rs
    ir/
      mod.rs
      process.rs
      node.rs
      edge.rs
      event.rs
      index.rs
    runtime/
      mod.rs
      instance.rs
      token.rs
      join.rs
      wait.rs
      dispatch.rs
      lifecycle.rs
    checkpoint/
      mod.rs
      model.rs
      codec.rs
      valkey.rs
      keys.rs
    dmn/
      mod.rs
      model.rs
      evaluate.rs
    host/
      mod.rs
      traits.rs
      types.rs
  tests/
    fixtures/
      bpmn/
```

Initial ownership intent:

1. `parser/` converts XML into normalized IR inputs
2. `ir/` holds immutable parsed and indexed graph structures
3. `runtime/` executes BPMN token semantics
4. `checkpoint/` owns durable state model and Valkey persistence
5. `dmn/` keeps explicit decision-binding placeholders for later DMN support
6. `host/` defines host-neutral contracts only

## 3. Minimal Public API

The v1 public API should stay small. The goal is to make `xiuxian-qianji`
consume the crate cleanly, not to publish a giant BPMN platform surface on day
one.

Even so, the crate should keep explicit DMN placeholder surfaces so later
decision support can be added without resetting the BPMN IR boundary.

### 3.1 Parse Surface

Illustrative parse API:

```rust
pub fn parse_bpmn_package(
    sources: &[BpmnSourceFile],
    options: &BpmnParseOptions,
) -> Result<BpmnPackage, BpmnEngineError>;
```

What this must do:

1. validate structural XML and BPMN invariants
2. normalize ids and imports
3. produce immutable package/process specs ready for runtime use

### 3.2 Instance Construction Surface

Illustrative instance API:

```rust
pub fn create_instance(
    package: std::sync::Arc<BpmnPackage>,
    process_id: &str,
    init: BpmnInstanceInit,
) -> Result<BpmnInstanceState, BpmnEngineError>;
```

What this must do:

1. bind one process spec from the parsed package
2. initialize runtime state
3. remain free of host-specific executor wiring

### 3.3 Drive Surface

Illustrative drive API:

```rust
pub async fn advance_instance<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
) -> Result<BpmnAdvanceOutcome, BpmnEngineError>;
```

The important point is the outcome shape, not the exact name. The runtime must
be able to report:

1. progressed internally
2. blocked on host work
3. blocked on external event
4. suspended
5. completed
6. failed

### 3.4 Checkpoint Surface

Illustrative checkpoint API:

```rust
pub async fn load_checkpoint(
    instance_id: &str,
    valkey_url: &str,
) -> Result<Option<BpmnCheckpointEnvelope>, BpmnEngineError>;

pub async fn save_checkpoint(
    checkpoint: &BpmnCheckpointEnvelope,
    valkey_url: &str,
) -> Result<(), BpmnEngineError>;
```

The public surface should load and save checkpoint envelopes, not raw JSON
strings.

## 4. Host Bridge Boundary

The host bridge should be thin and typed enough to keep runtime semantics out of
`xiuxian-qianji` while still letting the host provide real side effects.

### 4.1 Host Trait Responsibilities

The host trait should own:

1. dispatching service work
2. dispatching user/manual work
3. resolving external events or event delivery
4. providing current wall-clock time where timer semantics need it
5. optional telemetry hooks

Illustrative shape:

```rust
#[async_trait::async_trait]
pub trait BpmnHostBridge {
    async fn dispatch_service_task(
        &self,
        request: ServiceTaskRequest,
    ) -> Result<ServiceTaskOutcome, HostBridgeError>;

    async fn dispatch_user_task(
        &self,
        request: UserTaskRequest,
    ) -> Result<UserTaskOutcome, HostBridgeError>;

    async fn dispatch_manual_task(
        &self,
        request: ManualTaskRequest,
    ) -> Result<ManualTaskOutcome, HostBridgeError>;

    async fn poll_external_event(
        &self,
        request: EventPollRequest,
    ) -> Result<EventPollOutcome, HostBridgeError>;

    fn now_unix_ms(&self) -> u64;
}
```

### 4.2 What The Host Trait Must Not Own

The host trait must not:

1. decide gateway routing semantics
2. decide join completion semantics
3. mutate token frontier directly
4. deserialize or reshape checkpoint payloads
5. expose `xiuxian-qianji` scheduler internals to the BPMN crate

## 4.3 DMN Placeholder Rule

The crate should reserve a small DMN seam now even though DMN behavior remains
deferred.

Working rule:

1. keep a `src/dmn/` placeholder module
2. reserve a decision-reference slot on BPMN nodes that will later bridge to
   DMN-backed business-rule execution
3. avoid full DMN parsing or evaluation in the BPMN parser/runtime slices until
   there is an approved dedicated DMN slice

## 5. What Stays In xiuxian-qianji

`xiuxian-qianji` should implement the bridge and own:

1. Valkey URL resolution from runtime config
2. mapping BPMN service/user/manual requests to existing Qianji executors
3. CLI or app-facing orchestration commands
4. integration tests proving the bridge works
5. shared telemetry publication

`xiuxian-qianji` should not own:

1. BPMN IR types
2. BPMN runtime lifecycle enums
3. BPMN checkpoint key naming
4. BPMN suspend/resume serializer versions

## 6. First Scaffold Slice

The first code-bearing slice should still remain narrow.

### 6.1 In Scope

1. create the crate
2. add empty feature folders and `mod.rs`/`lib.rs` wiring
3. add error type shell
4. add type shells for `BpmnPackage`, `BpmnInstanceState`,
   `BpmnCheckpointEnvelope`, and `BpmnHostBridge`
5. add compile-only placeholder tests or type-level smoke tests

### 6.2 Out of Scope

1. XML parsing logic
2. BPMN execution loop
3. Valkey I/O implementation
4. timer semantics
5. Flowhub interoperability

## 7. Acceptance Gate For The First Scaffold

The first scaffold slice should be considered done only when:

1. the new crate exists in the workspace
2. dependency direction is one-way:
   `xiuxian-qianji -> qianji-bpmn-engine`
3. public API entrypoints compile
4. host trait shells compile
5. no BPMN runtime semantics leak into `xiuxian-qianji`
6. package docs and GTD/ExecPlan references stay synchronized

## 8. Audit Summary

This note fixes the crate-entry shape strongly enough for audit:

1. dedicated crate with feature-folder layout
2. small parse/instance/advance/checkpoint public API
3. thin host bridge trait
4. explicit list of what remains in `xiuxian-qianji`
5. a bounded first scaffold slice that still avoids real execution logic

## 9. Status Update After External Wait Slice

The crate boundary described above is now partially implemented beyond the
initial scaffold.

Current landed status:

1. parser/IR support exists for a bounded BPMN subset with dense node indices
   and adjacency lookup tables
2. `advance_instance` now executes the first bounded linear runtime subset:
   `startEvent`, `serviceTask`, `userTask`, `manualTask`, `businessRuleTask`
   host blocking or engine-owned local decision execution, and `endEvent`
3. service/user/manual tasks currently stop at explicit `PendingHostWork`
   boundaries instead of consuming host dispatch callbacks directly
4. `businessRuleTask` no longer depends on `DmnPlaceholder`; the package can
   now carry engine-owned DMN definitions for local execution, and the host
   seam remains available as a fallback
5. Valkey checkpoint persistence now exists as a real JSON state-key load/save
   surface
6. blocked service/user/manual tasks now have a typed host-result resume path
   through `apply_pending_host_work_result(...)`
7. blocked service/user/manual tasks now also expose typed dispatch requests
   through `PendingHostWorkRequest` and
   `build_pending_host_work_request(...)`
8. waiting instances now expose typed poll requests through
   `build_event_poll_request(...)` and bounded wait-outcome application through
   `apply_event_poll_outcome(...)`
9. the remaining near-term work should stay bounded around Valkey checkpoint
   sequence safety or thin `xiuxian-qianji` adapter surfaces rather than
   folding host execution into the BPMN runtime core

## 10. Status Update After DMN Contract Slice

The explicit DMN placeholder has now been tightened into a bounded crate-owned
contract without starting adapter work.

Current landed DMN status:

1. `qianji-bpmn-engine::dmn` now parses one DMN source into one decision with
   one decision table
2. the bounded evaluator now supports `UNIQUE` and `COLLECT` hit policies
3. supported input matching stays intentionally narrow: wildcard `-` plus
   literal equality for strings, numbers, booleans, and `null`
4. parser contract drift is now audited through an `insta` JSON snapshot over
   the parsed DMN decision definition
5. BPMN `businessRuleTask` runtime integration now has an engine-owned path,
   but production adapter wiring still remains for a later thin
   `xiuxian-qianji` slice

## 11. Status Update After Linter Contract Slice

The crate now also exposes a bounded lint contract aimed at future
`qianji lint --bpmn` and `qianji lint --dmn` adapter work.

Current landed linter status:

1. `qianji-bpmn-engine::lint` now exposes BPMN and DMN lint entrypoints
2. invalid BPMN and DMN sources are mapped into structured diagnostics instead
   of only raw parser errors
3. the lint report now includes explicit repair guidance and one
   `llm_fix_prompt` field so downstream CLI surfaces can hand the finding to an
   LLM without inventing a second diagnostic format
4. representative BPMN and DMN failure reports are snapshot-tested so later
   CLI adapter work inherits a stable contract
5. `xiuxian-qianji` currently only proves dependency linkage against the lint
   exports; the actual `qianji lint --bpmn/--dmn` CLI surface remains a later
   adapter slice

## 12. Status Update After Lint CLI Adapter Slice

The thin `xiuxian-qianji` adapter for the lint contract is now landed.

Current landed CLI status:

1. `qianji lint --bpmn <path>` now loads one BPMN source and renders the
   engine-owned lint report
2. `qianji lint --dmn <path>` now does the same for one DMN source
3. the CLI renderer preserves the engine-owned issue code, summary, repair
   guidance, and `llm_fix_prompt` instead of inventing a second diagnostic
   schema
4. clean sources exit with code `0`; blocking lint findings exit with code `2`
5. in the current worktree, the full historical `qianji` bin suite is still
   partially blocked by missing `qianji-flowhub` fixture content, but the new
   `lint` command tests and cross-crate dependency proof are green while
   `linter` remains a compatibility alias

## 13. Status Update After Business-Rule Host Seam Slice

The next bounded BPMN/DMN step is now landed on the engine side, but the host
adapter remains intentionally thin and deferred.

Current landed status:

1. BPMN `businessRuleTask` now blocks as typed pending host work instead of
   only suspending on `DmnPlaceholder`
2. the recoverable pending-work state now stores the DMN decision reference so
   a resumed instance can rebuild the same business-rule request deterministically
3. `qianji-bpmn-engine::host` now exports `BusinessRuleTaskRequest` and
   `BusinessRuleTaskOutcome` wrappers around the crate-owned
   `DmnEvaluationRequest` / `DmnEvaluationResult` contract
4. `apply_pending_host_work_result(...)` now resumes business-rule work
   through the same bounded host-result path used for service, user, and
   manual tasks
5. `xiuxian-qianji` currently proves only dependency linkage against these
   exports; there is still no production `BpmnHostBridge` implementation in
   the host crate
6. the next bounded adapter slice should therefore focus on host bridge
   wiring, not on widening DMN language support

## 14. Status Update After Parser Bundle Snapshot Slice

The parser can now populate the engine-owned DMN registry directly, while the
host adapter remains deferred.

Current landed status:

1. `qianji-bpmn-engine::parser` now exports `BpmnBundleSnapshot` and
   `parse_bpmn_bundle(...)`
2. the bounded bundle contract still requires exactly one BPMN source, but it
   can now carry zero or more DMN source snapshots alongside that BPMN source
3. bundled DMN sources are parsed through the crate-owned DMN parser and then
   attached to the returned `BpmnPackage` as registered decision definitions
4. invalid bundled DMN sources fail through the existing typed DMN parse
   errors instead of a second parser error surface
5. parser-owned registration now reaches the local `businessRuleTask`
   execution path without hand-built test-only package wiring
6. `xiuxian-qianji` still does not provide the production host bridge, so the
   next bounded move remains adapter wiring rather than further parser drift

## 15. Status Update After Xiuxian BPMN Adapter Slice

`xiuxian-qianji` now owns one thin production bridge layer for the bounded
host-work seam, while higher-level BPMN orchestration remains deferred.

Current landed status:

1. `packages/rust/crates/xiuxian-qianji/src/bpmn/adapter/` now owns a
   callback-backed `QianjiBpmnHostBridge` implementation under the host crate
2. unspecified service, user, manual, business-rule, and external-event
   callbacks fail through explicit `HostBridgeError::UnsupportedOperation`
   responses instead of silent no-op behavior
3. `dispatch_pending_host_work_request(...)`,
   `dispatch_pending_host_work_requests(...)`, and
   `resolve_pending_host_work(...)` now let `xiuxian-qianji` consume typed
   engine-owned pending host work and feed results back through
   `apply_pending_host_work_result(...)` plus `advance_instance(...)`
4. focused adapter tests now prove business-rule host completion and
   concurrent dispatch of one parallel service-task batch without moving BPMN
   token semantics into the host crate
5. full BPMN scheduler, CLI, or manifest-owned orchestration is still
   deferred, so the next bounded move is no longer bridge existence but
   higher-level runtime integration

## 16. Status Update After Xiuxian BPMN Orchestration Facade Slice

`xiuxian-qianji` now also owns one bounded host-side runtime/session facade on
top of the landed bridge, without pulling parser or runtime semantics back out
of `qianji-bpmn-engine`.

Current landed status:

1. `packages/rust/crates/xiuxian-qianji/src/bpmn/runtime/` now owns
   `QianjiBpmnSession`, `QianjiBpmnCheckpointStore`, and filesystem bundle-load
   helpers for one host-side orchestration entrypoint
2. `load_bpmn_package_from_files(...)` now builds one
   `qianji_bpmn_engine::BpmnBundleSnapshot`, reads bounded BPMN and DMN source
   files from disk, and delegates parsing back into
   `qianji-bpmn-engine::parse_bpmn_bundle(...)`
3. `QianjiBpmnSession` now keeps one immutable `Arc<BpmnPackage>` plus one
   mutable `BpmnInstanceState`, supports fresh instance construction, checkpoint
   resume, checkpoint export, and checkpoint persistence through the host-owned
   backend facade
4. checkpoint resume now guards process identity drift explicitly by comparing
   stored process package/spec identity against the loaded package before the
   host resumes execution
5. `run_until_stable(...)` now keeps advancement inside engine-owned semantics:
   it loops through `advance_instance(...)`, resolves typed host-blocked work
   through the landed bridge helpers, and stops only on stable waiting,
   suspended, completed, or failed outcomes
6. the host crate now exposes a bounded `sqlite` feature for lightweight local
   checkpoint storage while keeping distributed `Valkey` ownership as the
   default runtime path
7. focused tests now prove bundle loading with DMN registry attachment,
   automatic business-rule host completion through the new session facade,
   checkpoint identity-drift rejection, and `sqlite` checkpoint round-tripping
8. full BPMN scheduler integration, CLI execution ownership, and
   Flowhub/manifest convergence are still deferred, so the next bounded move
   should stay above this facade rather than reopening parser or DMN widening

## 17. Status Update After Qianji BPMN CLI Execution Slice

`qianji` now owns one bounded CLI execution surface on top of the landed
`xiuxian-qianji::bpmn::runtime` facade.

Current landed status:

1. `packages/rust/crates/xiuxian-qianji/src/bin/qianji.rs` now exposes
   `qianji bpmn run` as one explicit BPMN runtime entrypoint
2. the new command loads one BPMN bundle from disk, accepts zero or more DMN
   sidecar files, creates one fresh session or resumes one stored session, and
   drives that session until the next stable outcome
3. checkpoint backend selection now stays bounded and explicit: no backend by
   default, runtime-configured `Valkey` through `--checkpoint-runtime`, and one
   optional local `sqlite` path through `--checkpoint-sqlite <path>` when the
   `sqlite` feature is enabled
4. the command renders one stable markdown result surface including process,
   instance id, lifecycle, stable outcome, checkpoint provenance, and rendered
   workflow variables
5. the resume path now avoids stale checkpoint rewrites when a recovered
   waiting session makes no new progress in the current CLI invocation
6. focused binary-command tests now prove parse-contract behavior, one
   host-free linear completion path, and one waiting-session
   `sqlite` save/resume cycle
7. host callback injection, external event delivery, and scheduler-owned BPMN
   execution are still deferred, so the next bounded move should stay above
   this CLI surface rather than widening parser or DMN internals

## 18. Status Update After Fixture-Backed CLI Host Injection Slice

The same bounded CLI surface now also owns one deterministic host-injection
contract without widening engine ownership or scheduler responsibilities.

Current landed status:

1. `qianji bpmn run` now accepts `--host-fixture <path>` and loads one JSON
   contract from disk through the host crate rather than teaching the engine
   about fixture files
2. the fixture contract is keyed by stable BPMN node ids, so public CLI input
   does not expose dense internal `node_index` values from the engine runtime
3. the injected host bridge now covers bounded `sendTask`, `serviceTask`,
   `userTask`, `manualTask`, and `businessRuleTask` completion paths while
   preserving the existing explicit unsupported path for external-event
   delivery
4. the CLI renderer now reports the resolved host-fixture path in the run
   result so deterministic host injection stays auditable from the command
   output
5. focused binary-command tests now prove host-fixture parse behavior plus
   send-task, service-task, and business-rule completion through the CLI
   surface
6. scheduler-owned execution and real external-event wiring still remain
   deferred, so the next bounded move should stay above this deterministic
   host-fixture seam rather than reopening parser or DMN internals

## 19. Status Update After External Event Injection Slice

The host-owned BPMN facade now also consumes the engine wait-poll seam through
one bounded runtime and CLI layer.

Current landed status:

1. `packages/rust/crates/xiuxian-qianji/src/bpmn/adapter/wait.rs` now owns
   `resolve_waiting_external_event(...)` as the thin host-side counterpart to
   engine-owned `build_event_poll_request(...)` and
   `apply_event_poll_outcome(...)`
2. the helper preserves `WaitingExternalEvent` when the host leaves
   `poll_external_event(...)` unsupported, so default waiting behavior remains
   stable and explicit
3. `QianjiBpmnSession::run_until_stable(...)` now resolves one ready event and
   continues advancement inside the host facade without moving wait-routing
   semantics out of `qianji-bpmn-engine`
4. `qianji bpmn run` now accepts `--event-fixture <path>` and maps active BPMN
   wait ids to deterministic `poll_external_event(...)` outcomes without
   exposing internal runtime node indices
5. the CLI renderer now reports both `Host fixture` and `Event fixture` paths
   so deterministic injection stays auditable from the command output
6. focused library and binary tests now prove unsupported-poll preservation,
   ready-event resume through the session facade, and deterministic waiting-flow
   completion through the CLI surface
7. one bounded follow-up slice is now also landed above these deterministic
   host/event seams: waiting CLI output renders stable wait ids, competition
   gateway ids, wait metadata, and deterministic event-fixture keys for the
   active wait set
8. focused runtime and binary tests now also prove one explicit competing-wait
   contract end to end through `QianjiBpmnSession::run_until_stable(...)` and
   `qianji bpmn run --event-fixture`
9. one bounded reusable execution-driver slice is now also landed above these
   deterministic host/event seams, so later scheduler or app wiring can reuse
   package/session/checkpoint lifecycle without reopening CLI-local
   orchestration
10. focused runtime tests now also prove fresh execution, explicit
    fresh-context rejection, sqlite-backed checkpoint resume, and no-progress
    checkpoint-save skipping through the shared driver
11. that BPMN-specific scheduler checkpoint lifecycle slice is now also landed
    above this shared driver, with bounded checkpoint delete entrypoints plus
    one host-owned `QianjiBpmnExecutionScheduler`
12. terminal `Completed` or `Failed(...)` runs now delete persisted checkpoint
    state while waiting and suspended runs remain resumable through the same
    package/session/checkpoint seam
13. the active next bounded move is one Valkey-backed scheduler lease
    lifecycle so the host-owned BPMN scheduler becomes an explicit single
    writer for distributed checkpoint truth
14. that Valkey-backed scheduler lease lifecycle is now also landed, with
    owner-guarded save/delete plus explicit lease acquire/renew/release around
    one bounded scheduler execution attempt
15. generic DAG scheduler integration remains the follow-up slice after that
    BPMN-specific lifecycle step rather than reopening parser or DMN internals
16. that scheduler-identity bridge is now also landed above the BPMN lease
    lifecycle so host runtimes can derive explicit single-writer checkpoint
    ownership from `SchedulerAgentIdentity`
17. the landed bridge requires stable `agent_id` for BPMN lease ownership and
    keeps `role_class` outside the authoritative writer token contract
18. that CLI real-caller adoption slice is now also landed above the same
    bridge, so `qianji bpmn run` can reuse the BPMN scheduler lifecycle on the
    runtime-Valkey path without introducing a manual owner-token surface
19. the CLI adoption keeps role-only or absent identities on the existing
    driver path, while identity-backed runtime-Valkey runs now expose explicit
    checkpoint deletion in the rendered output
20. that runtime execution-selector slice is now also landed above the same
    bridge, so `xiuxian-qianji::bpmn::runtime` itself owns the driver-vs-
    scheduler choice through one reusable `QianjiBpmnExecutionFacade`
21. `qianji bpmn run` is now only a thin adapter over that library selector,
    while future host runtimes can adopt the same facade directly
22. higher-level host runtime adoption beyond the CLI remains the follow-up
    slice after that selector, while generic DAG scheduler integration still
    stays deferred rather than reopening parser or DMN internals
23. one bounded CLI gate-cleanup slice is now also landed on top of that
    adapter surface: the bloated `tests/unit/bin/qianji/bpmn.rs` and
    `tests/unit/bpmn/runtime.rs` files are now folder-first suites with shared
    support helpers rather than single large files
24. the `qianji` binary root is now a thin entry file, while BPMN, dir, lint,
    and contract-feedback concerns delegate into `src/bin/qianji/` feature
    files
25. the exact crate test-policy harness is green again for `xiuxian-qianji`
    after that structural cleanup
26. focused BPMN library tests plus both `qianji` BPMN binary suites (`llm`
    and `llm sqlite`) remain green after the refactor
27. the exact modularity harness is still not green: the old oversized root
    file is gone, but the crate still needs a follow-up seam-cleanup pass for
    the new binary root/facade layout and other pre-existing crate findings
28. one bounded follow-up seam-cleanup cut is now also landed under
    `src/contracts/flowhub/`: the folder has a single owner-style `api.rs`
    seam with explicit sibling imports instead of a multi-surface root facade
29. the heavier `MOD-R008` / `MOD-R018` / `MOD-R019` findings that previously
    targeted `src/contracts/flowhub/mod.rs` are now gone, although one
    narrower visible-owner follow-up remains on that folder
30. the imported `qianji-flowhub` contract integration tests have also been
    refreshed to current live `plan` and `research/paper` manifest reality, so
    host-crate validation no longer depends on older localized-workdir
    assumptions for those specific fixtures
31. one further bounded cleanup cut is now also landed at the
    `src/contracts/` root: `contracts/api.rs` is now the single visible owner
    seam for public contract exports, and the temporary helper-bucket
    experiment was removed again
32. `src/contracts/mod.rs` no longer appears in the exact modularity output,
    so the remaining `contracts`-cluster gate work has collapsed to the nested
    owner seams in `flowhub/` and `workdir/` plus later visibility narrowing
    of the contract structs themselves
33. one further bounded cleanup cut is now also landed at the BPMN host-adapter
    surface: `src/bpmn/mod.rs` and `src/bpmn/runtime/` now both route through
    explicit `api.rs` owner seams instead of multi-surface root facades
34. the exact modularity output no longer calls out `src/bpmn/mod.rs`, and the
    previous `src/bpmn/runtime/mod.rs` root-facade findings have collapsed to
    one narrower visible-owner seam plus later visibility tightening on the
    runtime structs themselves
35. one further BPMN seam-cleanup cut is now also landed above that runtime
    experiment: the host crate no longer keeps `src/bpmn/runtime/mod.rs` in
    the active module graph
36. instead, `src/bpmn/mod.rs` now mounts the `runtime/*.rs` owners directly
    through private path-based declarations, and `src/bpmn/api.rs` re-exports
    from those leaf owners while the physical `runtime/` folder remains the
    on-disk grouping boundary
37. the filtered BPMN modularity output therefore no longer reports
    `src/bpmn/runtime/mod.rs`; the remaining BPMN-cluster findings are the
    internal `adapter/` seam plus later `MOD-R002` visibility narrowing on the
    runtime leaf owners
38. one matching cleanup cut is now also landed for the adapter side of the
    same feature: the host crate no longer keeps `src/bpmn/adapter/mod.rs` in
    the active module graph either
39. `src/bpmn/mod.rs` now mounts the adapter leaf owners directly as private
    path-based declarations too, with a root-safe `adapter_error` mount for
    the adapter-local error file
40. the filtered BPMN modularity output therefore no longer reports either
    internal BPMN `mod.rs` facade; the remaining BPMN findings are leaf-level
    visibility-tightening follow-ups on the adapter and runtime owners
41. one further bounded BPMN gate-cleanup cut is now also landed on those
    remaining owner files: the host crate has relocated each public-surface
    BPMN leaf owner from `src/bpmn/adapter/*.rs` and `src/bpmn/runtime/*.rs`
    into direct `src/bpmn_*.rs` files
42. `src/bpmn/mod.rs` still owns the feature boundary as the interface-only
    table of contents, but its private path mounts now point at the direct
    `src/` owner files so the public BPMN surface no longer lives under nested
    leaf paths that trigger the current modularity visibility rule
43. the filtered modularity probe for `rg 'src/bpmn(/|_)'` is now empty after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib bpmn:: --features "llm sqlite"`, and
    `git diff --check`, so the remaining modularity backlog is now outside the
    BPMN feature boundary
44. one follow-up gate-cleanup cut is now also landed on the nested contracts
    owners used by the broader host crate: the Flowhub/workdir contract owners
    that previously lived under `src/contracts/flowhub/*` and
    `src/contracts/workdir/*` have been relocated into direct
    `src/contracts_*` owner files
45. the internal `flowhub/mod.rs` + `flowhub/api.rs` and `workdir/mod.rs` +
    `workdir/api.rs` seams are therefore no longer part of the active source
    tree, while `contracts/api.rs` and `contracts/mod.rs` still preserve one
    stable top-level contract entry seam
46. the exact filtered modularity probe for
    `src/contracts/flowhub|src/contracts/workdir|src/contracts_flowhub_|src/contracts_workdir_`
    is now empty after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test flowhub_manifest_contracts --features "llm sqlite"`,
    and `git diff --check`
47. one smaller follow-up cut is now also landed for the remaining
    app/bootstrap public-owner hotspots that were still blocking `MOD-R002`:
    `app/presets.rs`, `app/qianji_app.rs`, `bootcamp/model.rs`, and
    `bootcamp/workflow.rs` now live as direct `src/app_*.rs` and
    `src/bootcamp_*.rs` owner files
48. `app/mod.rs` and `bootcamp/mod.rs` remain the stable interface seams, but
    their private path mounts now point at the relocated direct-`src` owners so
    the filtered modularity probe for `src/app(/|_)|src/bootcamp(/|_)` is now
    also empty after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib bootcamp:: --features "llm sqlite"`,
    and `git diff --check`
49. one further bounded gate-cleanup cut is now also landed for the
    `consensus` surface: `consensus/models.rs`, `consensus/manager/mod.rs`,
    and `consensus/manager/voting/mod.rs` have been replaced by direct
    `src/consensus_*.rs` owner files while `src/consensus/mod.rs` remains the
    interface-only table of contents
50. the same cut also removed one stray
    `src/consensus/manager/voting/vote_store.rs` file after confirming it was
    an unmounted byte-for-byte duplicate of the real
    `src/swarm/discovery/registry/heartbeat.rs` implementation rather than a
    live consensus owner
51. the exact filtered modularity probe for `src/consensus(/|_)` is now empty
    after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test integration_test test_consensus --features "llm sqlite"`,
    and `git diff --check`, so the remaining gate backlog has moved on again
    to other seams outside the consensus feature boundary
52. one further bounded gate-cleanup cut is now also landed for the
    `contract_feedback` feature: the public owners that previously lived at
    `contract_feedback/pipeline.rs` and `contract_feedback/rest_docs.rs` now
    live as direct `src/contract_feedback_*.rs` files
53. `contract_feedback/mod.rs` remains the stable interface seam, but its
    private path mounts now point at the relocated direct-`src` owners so the
    filtered modularity probe for `src/contract_feedback(/|_)` is now empty
    after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib contract_feedback:: --features "llm sqlite"`,
    and `git diff --check`
54. one further bounded gate-cleanup cut is now also landed for the remaining
    top-level `contracts` owners: `bindings.rs`, `execution.rs`, `manifest.rs`,
    `mechanism.rs`, and `wendao_docs.rs` now live as direct
    `src/contracts_*.rs` files while `contracts/mod.rs` stays the folder-level
    entry seam
55. the former oversized `contracts_wendao_docs.rs` owner has also been
    reduced to one thin root seam with private internal helper files under
    `src/contracts_wendao_docs/`, so the contract display and invocation
    validation behavior remains stable without leaving one 300+ line owner file
    in the active source graph
56. the exact filtered modularity probe for `src/contracts(/|_)` is now empty
    after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib contracts:: --features "llm sqlite"`,
    and `git diff --check`, so the remaining gate backlog has moved on again
    to later engine, scheduler, swarm, telemetry, sovereign, and workdir seams
57. one further bounded compiler cleanup cut is now also landed for the first
    `engine/compiler` failure cluster: the internal `leaf_dispatch/mod.rs` and
    `stateful_cfg/mod.rs` seams under `mechanism_dispatch` have been replaced
    by direct `src/engine_compiler_mechanism_dispatch_*.rs` owner files
58. `engine/compiler/mechanism_dispatch.rs` now stays as the stable local seam
    with one short root doc hint, while its private path mounts point at the
    relocated direct-`src` owners so the filtered modularity probe for
    `src/engine/compiler/mechanism_dispatch|src/engine_compiler_mechanism_dispatch_`
    is now empty after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib engine::compiler:: --features "llm sqlite"`,
    and `git diff --check`
59. one further bounded compiler cleanup cut is now also landed at the
    compiler root: `QianjiCompiler` no longer lives in `compiler/mod.rs`, and
    the `task_mechanisms` plus `wendao_sql` internal facades no longer rely on
    nested `mod.rs` files either
60. instead, `compiler/mod.rs` is back to one interface seam with a short root
    hint aligned to `api`, while the real owners now live in
    `src/engine_compiler_api.rs`,
    `src/engine_compiler_task_mechanisms.rs`, and
    `src/engine_compiler_wendao_sql.rs`
61. the exact filtered modularity probe for
    `src/engine/compiler/mod.rs|src/engine/compiler/task_mechanisms|src/engine/compiler/wendao_sql|src/engine_compiler_api|src/engine_compiler_task_mechanisms|src/engine_compiler_wendao_sql`
    is now empty after `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib engine::compiler:: --features "llm sqlite"`,
    and `git diff --check`, so the next backlog slice has moved on again to
    later scheduler, swarm, telemetry, sovereign, and workdir seams
62. one further bounded cleanup cut is now also landed for the old
    `scheduler/*` cluster: the nested `checkpoint`, `identity`, `policy`,
    `state`, and `preflight` owners no longer sit under `src/scheduler/`,
    because the live owners now sit at direct `src/scheduler_*.rs` paths while
    `scheduler/mod.rs` has collapsed back to one thin public facade
63. the same scheduler cut also replaced the former
    `scheduler/core/types/*` seam with direct `src/scheduler_core_types.rs`
    and `src/scheduler_execution.rs` owners, then reduced the remaining
    `scheduler/core/*` roots so that `consensus/resolve/mod.rs` and
    `telemetry/mod.rs` are interface-only again and the `remote_possession` /
    `run_loop` roots now give an explicit first-hop hint
64. the exact filtered modularity probe for
    `src/scheduler(/|_)|scheduler/mod.rs|scheduler/core/` is now empty after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib scheduler:: --features "llm sqlite"`,
    and `git diff --check`, so the active modularity backlog has now moved on
    again to later non-scheduler seams
65. the shared `xiuxian-testing` modularity gate is also now more operator- and
    LLM-friendly during the remaining cleanup work: blocking reports include
    `why:` plus `fix:` guidance, and the relevant root-seam rules now state
    the private `mod` / private `#[path = "..."] mod` plus selective
    re-export pattern explicitly instead of leaving that refactoring pattern
    implicit
66. one further bounded cleanup cut is now also landed for the old
    `sovereign/*` cluster: the nested owner files no longer sit under
    `src/sovereign/`, because the live owners now sit at direct
    `src/sovereign_*.rs` paths while `sovereign/mod.rs` has collapsed back to
    one thin facade
67. that sovereign cut also reduced the public root seam to the
    contract-feedback persistence types actually consumed by current host and
    integration callers, while the support owners used only by focused tests
    now mount under `#[cfg(test)]` so touched-scope `cargo check` stays free
    of dead-code warnings
68. the exact filtered modularity probe for
    `src/sovereign(/|_)|sovereign/mod.rs` is now empty after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib sovereign:: --features "llm sqlite"`,
    and `git diff --check`, so the active modularity backlog has now moved on
    again to the remaining `swarm/*` seams
69. one further bounded cleanup cut is now also landed for the old
    `swarm/discovery/*` cluster: the nested `model.rs`, `registry/mod.rs`,
    `parse.rs`, and `util.rs` seams no longer sit under `src/swarm/discovery/`,
    because the live owners now sit at direct `src/swarm_discovery_*.rs`
    paths while the remaining `registry/*` files act only as helpers behind
    the direct registry owner
70. that discovery cut also removed the internal `swarm/discovery/mod.rs`
    seam entirely and moved the public API re-exports up to `swarm/mod.rs`, so
    the stable external symbols remain `ClusterNodeIdentity`,
    `ClusterNodeRecord`, and `GlobalSwarmRegistry` without relying on a
    visible internal discovery facade
71. the exact filtered modularity probe for
    `src/swarm/discovery(/|_)|src/swarm_discovery_` is now empty after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test integration_test test_swarm_discovery --features "llm sqlite"`,
    and `git diff --check`, so the active modularity backlog has now moved on
    again to the remaining non-discovery `swarm/*` seams
72. one further bounded cleanup cut is now also landed for the old
    `swarm/engine/*` cluster: the public engine owners no longer sit under
    `src/swarm/engine/orchestrator.rs` or
    `src/swarm/engine/types/{agent,options,report}.rs`, because the live
    owners now sit at direct `src/swarm_engine_orchestrator.rs` and
    `src/swarm_engine_types.rs` paths
73. `swarm/mod.rs` now re-exports the stable public engine API from those
    direct owners, while `src/swarm/engine/mod.rs` has collapsed to a private
    worker namespace and the `swarm/engine/worker/*` leaves now import the
    direct owner modules privately instead of relying on another visible root
    facade
74. the exact filtered modularity probe for
    `src/swarm/engine(/|_)|src/swarm_engine_` is now empty after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test integration_test test_swarm_orchestration --features "llm sqlite"`,
    and `git diff --check`, so the active modularity backlog has now moved on
    again to the remaining top-level `swarm/mod.rs` plus `swarm/possession/*`
    seams
75. one further bounded cleanup cut is now also landed for the old
    `swarm/possession/*` cluster: the public possession bus, remote node
    request/response model, and execution-error mapping owners no longer sit
    under `src/swarm/possession/`, because the live owners now sit at direct
    `src/swarm_possession_*.rs` paths
76. the remaining `swarm/possession/bus/{connection,keys,request,response}.rs`
    files now act only as private helper leaves behind the direct possession
    bus owner, and the public bus methods no longer live on those helper files
77. the exact filtered modularity probe for
    `src/swarm/possession(/|_)|src/swarm_possession_` is now empty after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test integration_test test_swarm_orchestration --features "llm sqlite"`,
    and `git diff --check`
78. the final top-level `swarm/mod.rs` export seam is now also narrowed
    through one direct `api` owner (`src/swarm_api.rs`), so the root no
    longer re-exports public symbols from many child surfaces directly
79. the exact filtered modularity probe for `src/swarm(/|_)|src/swarm_` is
    now empty after the same `cargo check`, targeted `test_swarm_orchestration`,
    and `git diff --check` validation, so the active gate backlog has moved on
    again to non-swarm seams
80. one further bounded cleanup cut is now also landed for the old
    `engine` root plus `engine/compiler` root cluster: the public `QianjiCompiler`
    owner no longer depends on a visible `pub mod compiler;` seam under
    `src/engine/mod.rs`, because the live root owners now sit at direct
    `src/engine_api.rs` and `src/engine_compiler_api.rs` paths
81. that engine-root cut also reduced `src/engine/compiler/mod.rs` to one
    canonical internal owner (`pipeline`), while the new direct
    `src/engine_compiler_pipeline.rs` owner now carries the restricted
    parse-assemble-dispatch-validate flow above the private compiler helper
    leaves
82. the exact filtered modularity probe for
    `src/engine/mod.rs|src/engine_api.rs|src/engine/compiler(/|_)|src/engine_compiler_`
    is now empty after `cargo fmt --manifest-path packages/rust/crates/xiuxian-qianji/Cargo.toml`,
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test integration_test test_compiler_dispatch_routes --features "llm sqlite"`,
    and `git diff --check`, so the active backlog has moved on again to the
    next non-engine seam
83. one further bounded cleanup cut is now also landed for the old
    `executors` root cluster: the public built-in mechanism surface no longer
    depends on `executors::annotation::*`, `executors::formal_audit::*`,
    `executors::llm::*`, or `executors::wendao_sql::*` as the first hop,
    because the live root now points at one direct `src/executors_api.rs`
    owner and exposes `executors::TypeName` entries instead
84. that executors cut also removed the internal feature-folder root seams
    from the active source tree: the live owners now sit at direct
    `src/executors_annotation.rs`, `src/executors_formal_audit.rs`,
    `src/executors_llm.rs`, `src/executors_security_scan.rs`,
    `src/executors_wendao_ingester.rs`, `src/executors_wendao_refresh.rs`,
    `src/executors_wendao_sql.rs`, and `src/executors_write_file.rs` paths,
    while the replaced `src/executors/*/mod.rs` files were deleted
85. the exact filtered modularity probe for
    `src/executors/mod.rs|src/executors_api.rs|src/executors_(annotation|formal_audit|llm|security_scan|wendao_ingester|wendao_refresh|wendao_sql|write_file).rs|src/executors/.*/mod.rs`
    is now empty after `cargo fmt --manifest-path packages/rust/crates/xiuxian-qianji/Cargo.toml`,
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test integration_test test_qianji_trinity_integration --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --lib executors:: --features "llm sqlite"`,
    and `git diff --check`, so the active backlog has moved on again to the
    next non-executors seam
86. one further bounded cleanup cut is now also landed for the old
    `flowhub/materialize` internal seam: `flowhub/mod.rs` no longer mounts a
    nested `materialize/mod.rs` facade, because the live `flowhub` root now
    privately path-mounts `materialize/{anchored,copy,root,safety,scenario}.rs`
    directly and exposes the public materialize surface through one direct
    `src/flowhub_materialize_api.rs` owner
87. that materialize cut also removed the replaced
    `src/flowhub/materialize/mod.rs` and `src/flowhub/materialize/api.rs`
    files from the active source tree, so the touched leaves now import the
    direct `flowhub` private owners instead of routing through another
    internal root facade
88. the live imported `qianji-flowhub/research/paper` contract has drifted
    again: the manifest still resolves the deep-read scenario through
    `paper-deep-read`, but the current live `paper-deep-read.mmd` graph no
    longer declares a localized check surface. The real-anchor
    `flowhub_materialize` integration tests now gate those assertions on
    `show_flowhub_graph(...).declared_check_surface.root.is_some()` so the
    suite stays aligned to current imported contract reality without weakening
    the fixture-backed materialize behavior coverage
89. the exact filtered modularity probe for
    `src/flowhub/mod.rs|src/flowhub_materialize_api.rs|src/flowhub/materialize/(anchored|copy|root|safety|scenario).rs|src/flowhub/materialize/mod.rs|src/flowhub/materialize/api.rs`
    no longer reports the deleted `src/flowhub/materialize/mod.rs` seam or
    the new direct `src/flowhub_materialize_api.rs` owner after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test flowhub_materialize --features "llm sqlite"`,
    and `git diff --check`; the remaining flowhub backlog is now the broader
    `src/flowhub/mod.rs` root plus leaf-level `MOD-R002` visibility
    tightening
90. one further bounded cleanup cut is now also landed for the old
    `flowhub/mermaid` internal seam: `flowhub/mod.rs` no longer exposes a
    visible `pub(crate) mod mermaid;`, because the live root now privately
    path-mounts `mermaid/{model,parse,topology,validate}.rs` directly and
    routes internal Mermaid helper access through one direct
    `src/flowhub_mermaid_api.rs` owner
91. that Mermaid cut also removed the replaced
    `src/flowhub/mermaid/mod.rs` file from the active source tree, and the
    touched Flowhub / workdir / unit-test call sites now import Mermaid
    helpers from the `flowhub` root internal seam instead of
    `flowhub::mermaid::*`
92. the bounded Mermaid validation slice also refreshed the live graph-show
    expectations for `plan/codex-plan.mmd`,
    `research/paper/paper-canonicalize.mmd`, and
    `research/paper/paper-deep-read.mmd`: those imported graphs no longer
    declare `[graph.workdir]`, so the live `show_flowhub_graph(...)`
    assertions now align to the current no-bounded-check-surface renderer
    contract instead of older localized-workdir expectations
93. the exact filtered modularity probe for
    `src/flowhub/mod.rs|src/flowhub_mermaid_api.rs|src/flowhub/mermaid/(model|parse|topology|validate).rs|src/flowhub/mermaid/mod.rs`
    no longer reports the deleted `src/flowhub/mermaid/mod.rs` seam or the
    new direct `src/flowhub_mermaid_api.rs` owner after
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji mermaid --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji --test flowhub_materialize --features "llm sqlite"`,
    and `git diff --check`; the next remaining flowhub root blocker is the
    visible `scenario_ir` declaration plus the still-wide public export
    surface
94. one further bounded cleanup cut is now also landed for the old
    `flowhub/scenario_ir` leaf cluster: the live `annotations.rs` and
    `compile.rs` owners keep the same `flowhub`-internal surface, but the
    heavy parsing and compile logic now lives behind private helper owners
    `annotation_{model,node,support}.rs` and
    `compile_{legacy,nodes,workdir}.rs`
95. that `scenario_ir` cut keeps the owner seams narrow: the current
    `annotations.rs` leaf is now just the annotation error plus parse
    orchestration, while the current `compile.rs` leaf is now just graph-name
    resolution plus top-level scenario-IR assembly; the existing
    `flowhub_scenario_ir_api.rs` seam still carries the stable internal
    re-export surface above them
96. the bounded validation slice for this cut stayed inside the same flowhub
    frontier and kept one directly affected integration proof in the loop:
    `cargo check -p xiuxian-qianji --features "llm sqlite"`,
    `cargo test -p xiuxian-qianji scenario_ir --features "llm sqlite"`, and
    `cargo test -p xiuxian-qianji --test flowhub_materialize --features "llm sqlite"`
97. the exact filtered modularity probe for
    `src/flowhub/mod.rs|src/flowhub_api.rs|src/flowhub_scenario_ir_api.rs|src/flowhub/scenario_ir/(annotation_model|annotation_node|annotation_support|annotations|compile|compile_legacy|compile_nodes|compile_workdir|model).rs|src/flowhub/scenario_ir/mod.rs`
    is now empty after the same validation pass and `git diff --check`, so
    the active backlog has moved on again to the next non-`flowhub` seam
98. the next highest-signal non-`flowhub` root error in the exact modularity
    harness was `src/layout/mod.rs`, so one further bounded cleanup cut is now
    also landed for the `layout` cluster instead of reopening several
    unrelated seams at once
99. that `layout` cut relocated the live public owners to direct `src/`
    seams: `layout/mod.rs` now privately path-mounts
    `src/layout_{api,bpmn,engine,engine_types,style}.rs`, while the replaced
    nested owner files under `src/layout/` and the stale duplicate
    `src/layout/engine/deep_graph.rs` were deleted from the active source
    tree
100. the root `layout` API stayed stable through `src/layout_api.rs`: callers
     still reach `QianjiLayoutEngine`, `QgsTheme`, `generate_bpmn_xml`, and
     the layout payload types from `xiuxian_qianji::layout`, but the root seam
     itself is now interface-only instead of exposing visible nested
     `pub mod` facades
101. the exact filtered modularity probe for
     `src/layout/mod.rs|src/layout_api.rs|src/layout_(bpmn|engine|engine_types|style).rs|src/layout/(bpmn|style).rs|src/layout/engine/(mod|deep_graph|layout_core|types).rs`
     is now empty after
     `cargo check -p xiuxian-qianji --features "llm sqlite"`,
     `cargo test -p xiuxian-qianji test_omg_standard_compliance_branching_flow --features "llm sqlite"`,
     and `git diff --check`, so the active backlog has moved on again to the
     next remaining seam
