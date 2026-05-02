---
type: knowledge
title: "Design Note: qianji-bpmn-engine Frontier Concurrency and Synchronization Semantics"
category: "research"
status: "draft"
authors:
  - codex
created: 2026-04-19
tags:
  - qianji
  - bpmn
  - concurrency
  - gateway
  - runtime
  - omg
---

# Design Note: qianji-bpmn-engine Frontier Concurrency and Synchronization Semantics

## 1. Purpose

This note narrows the next bounded `qianji-bpmn-engine` slice to one question:

How should the engine align its runtime model with OMG BPMN concurrency and
synchronization semantics when a single workflow instance has multiple runnable
nodes at the same time?

The key distinction for this lane is:

1. BPMN semantic concurrency is required
2. multi-writer checkpoint concurrency is not

This note therefore does not argue for multiple distributed writers. It argues
for a better in-instance frontier model under the existing single-writer
ownership contract.

Companion notes:
[Research Plan: qianji-bpmn-engine Architecture and xiuxian-qianji Integration](2026-04-18-bpmn-engine-research-plan.md)
and
[Design Note: qianji-bpmn-engine Runtime State and Valkey Checkpoint Model](2026-04-18-bpmn-runtime-state-and-valkey-checkpoint-design.md)
and
[Audit Note: qianji-bpmn-engine BPMN and DMN Parity Against SpiffWorkflow](2026-04-18-bpmn-dmn-spiff-parity-audit.md)

Primary normative source:
[OMG BPMN 2.0.2 Specification](https://www.omg.org/spec/BPMN/2.0.2/PDF)

## 2. Normative BPMN Semantics That Matter for This Slice

### 2.1 Parallel Gateway

OMG BPMN 2.0.2 Clause 13.4.1 defines the parallel gateway as both:

1. a branching point that spawns concurrent branches
2. a merge point that synchronizes concurrent branches

For `qianji-bpmn-engine`, the important operational consequences are:

1. a parallel split is not just graph fan-out; it creates multiple active
   concurrent branches
2. a parallel join waits until there is at least one token on every incoming
   sequence flow
3. when the join fires, it consumes one token per incoming sequence flow and
   emits one token on each outgoing sequence flow
4. excess tokens are not destroyed just because the join fired once; they must
   remain represented in runtime state

This last point is important because it means the engine cannot model parallel
join semantics only as a boolean "ready/not ready" flag. It needs token-aware
frontier bookkeeping.

### 2.2 Event-Based Gateway

OMG BPMN 2.0.2 Clause 10.6.6 defines the event-based gateway as a branching
point where routing depends on which event happens, not on data expressions.

For this engine, the key consequences are:

1. the outgoing sequence flows do not carry data conditions
2. the runtime must register multiple event waits as one competition
3. one event wins the race and the losing alternatives are cancelled
4. this is semantic concurrency at the wait frontier even when only one branch
   ultimately continues

This matches the existing bounded event-competition direction, but it also
means a future frontier model must treat wait registration as a set owned by
one gateway, not as unrelated singleton waits.

### 2.3 Parallel Event-Based Gateway

The BPMN 2.0.2 spec distinguishes the parallel event-based gateway from the
ordinary event-based gateway and constrains it to process instantiation.

For the current bounded engine, the implication is:

1. do not invent a generic mid-process "parallel event race" runtime mode
2. keep the currently supported in-process event-based gateway semantics
   exclusive
3. if mid-process behavior truly needs parallel event handling, model it with
   standard parallel control-flow constructs plus explicit waits, not with a
   misread gateway type

## 3. Engine Implications

### 3.1 Semantic Concurrency Must Live Inside One Writer

The existing checkpoint design remains correct:

1. one workflow instance has one distributed writer owner at a time
2. Valkey lease ownership is about checkpoint truth and stale-writer exclusion
3. semantic concurrency happens inside that owner process as runtime state,
   not by allowing multiple checkpoint writers

This preserves deterministic resume and aligns with the existing Valkey-first
runtime-state direction.

### 3.2 Current Runtime Gap

The current engine already carries several structures that prove it is not
single-token in principle:

1. `active_tokens`
2. `joins`
3. `waits`
4. `event_competition`

However, the bounded runtime still advances around a first-token planning model
and therefore still lacks an explicit frontier proposal/reduce phase.

That means the engine currently has:

1. token-scoped ownership for multiple blocked host-work entries
2. token-scoped runnable selection so duplicate tokens at the same BPMN node do
   not get hidden behind one shared node-status bit
3. edge-aware buffered join arrivals so parallel joins do not fire early when
   duplicate arrivals come from the same incoming sequence flow
4. an explicit `BpmnFrontierSnapshot` that classifies every active token into
   deterministic frontier states such as runnable, blocked-on-host, or
   waiting-external
5. explicit frontier proposal collection that can surface every runnable token
   in deterministic token order
6. explicit deterministic reduction from those proposals to one owner action
   such as execute-batch, blocked-on-host, waiting-external, suspended, or
   stalled
7. deterministic batch consumption that re-resolves tokens by `token_id`
   before each in-batch mutation so stale snapshot indices do not misapply
   later proposals
8. shifted-index resolution for common prefix-removal batches, so a wide
   frontier whose runnable tokens disappear in deterministic order does not
   fall back to repeated linear `token_id` scans; the focused probe with
   `stable_prefix=4000`, `removable_tokens=4000`, and `iterations=8` measured
   `linear_ms=422.663` versus `shifted_cursor_ms=47.669`
9. one bounded conflict-aware cross-token merge model is now landed for
   same-node parallel joins, but broader node-family merge remains open

### 3.3 The Required Runtime Shape

The next architectural target should deepen that frontier-based model beyond
the landed proposal/reduction and batch-consumption seams:

1. collect all runnable token positions for the current instance
2. plan per-token transition proposals against immutable process specs
3. reduce those proposals deterministically into one owner batch
4. materialize host-dispatch work and wait registrations from the consumed
   frontier result
5. checkpoint only after the owner has one coherent post-step instance state

This keeps semantic concurrency explicit while preserving a single-writer
checkpoint truth model.

## 4. What This Means for `rayon`

`rayon` is now appropriate only for pure frontier inspection work, not for
state mutation.

The safe layering is:

1. semantics first
2. deterministic frontier planning second
3. optional CPU-parallel planning third

That means:

1. immutable frontier classification may use `rayon` when the active-token set
   is wide enough to amortize scheduling overhead
2. host-dispatch I/O should stay async
3. Valkey checkpoint and lease I/O must stay async and single-writer
4. correctness must not depend on thread scheduling order

## 5. Concrete Bounded Cases to Carry Into Tests

The next runtime slice should prove at least these cases.

### Case 1. Parallel Split with Two Host-Blocking Branches

1. one parallel gateway fans out into two leaf tasks
2. both branches become active in the same instance frontier
3. the runtime does not collapse them into one singleton pending-work slot

### Case 2. Parallel Join Synchronization

1. two branches rejoin at one parallel gateway
2. the join does not fire early
3. one token per incoming branch is consumed when the join fires

### Case 3. Event-Based Gateway Race

1. one event-based gateway fans out into multiple catch-event waits
2. the runtime records them as one competition
3. one winner continues and losing waits are cancelled deterministically

### Case 4. Excess-Token Join Behavior

1. if a join sees extra tokens on one incoming branch, they are not silently
   deleted by one join firing
2. the runtime state must still be able to represent post-fire excess token
   presence

### Case 5. Keep Multi-Instance Separate

1. gateway concurrency and multi-instance concurrency are not the same family
2. parallel multi-instance stayed out of scope for the original frontier
   slice, and should continue to be modeled as a separate owner-state family
   rather than as a gateway-special case
3. this slice should not overload gateway-frontier logic with full
   multi-instance expansion semantics

## 6. Recommended Next Slice

The next bounded implementation slice after the landed frontier
proposal/reduction seam was:

1. keep single-writer checkpoint ownership unchanged
2. keep the landed token-scoped blocked/runnable ownership, explicit frontier
   snapshots, explicit proposal collection, deterministic reduction, and
   edge-aware join buffering as the new baseline
3. widen runtime planning from reduce-to-one-owner-step into deterministic
   multi-proposal batch execution over multiple runnable tokens
4. re-resolve proposals by stable `token_id` ownership so in-batch token
   removal and index movement stay correct

The next bounded slice after that is now landed:

1. single-writer checkpoint ownership stayed unchanged
2. deterministic batch execution remained the baseline
3. execution still keeps one bounded conflict-aware merge for same-node
   parallel joins
4. the batch path now adds one batch-local token lookup seam so repeated
   proposal and join resolution no longer depends on repeated linear
   `token_id -> active_tokens` scans
5. the lifecycle state seam also split into focused state lookup, token, and
   join siblings so the modularity gate stays green without changing runtime
   semantics
6. the ignored local probe for this hotspot measured
   `linear_ms=817.135` versus `batch_lookup_ms=113.153` over
   `tokens=10000`, `lookups_per_batch=512`, and `iterations=64`
7. adapter work, DMN widening, and inclusive gateways remained out of scope
8. bounded parallel multi-instance later landed as a separate owner-state
   slice on top of the same deterministic batch-execution baseline

The next bounded follow-up after the landed token-lookup slice should be:

1. keep single-writer checkpoint ownership unchanged
2. keep raw runnable proposal collection plus merge-aware batch execution as
   the new baseline
3. widen conflict-aware frontier merge only when another BPMN node family
   proves it needs aggregate semantics
4. keep adapter work, DMN widening, inclusive gateways, and parallel
   multi-instance out of scope

## 7. Final Design Stance

The correct architectural reading is:

1. BPMN requires multiple active nodes inside one workflow instance
2. `qianji-bpmn-engine` therefore needs a frontier-aware runtime model
3. the model now includes one bounded conflict-aware merge for same-node
   parallel joins on top of the earlier snapshot/planner/proposal/batch seams
4. this does not justify multiple checkpoint writers
5. each future merge rule should still be justified by BPMN semantics first,
   with `rayon` remaining limited to pure planning work

## 8. Follow-up After Event Competition Wait Slice

The next runtime-local hot path after the landed frontier token-lookup work is
now also closed without widening BPMN semantics.

What landed:

1. event-based gateway competition settlement now uses one temporary indexed
   membership context when the wait fan-out is wide enough, so the same wait
   owner no longer pays repeated `Vec::contains` scans across token compaction
   and wait cleanup
2. winner-token settlement now completes in one retention pass instead of
   compacting `active_tokens` and then rescanning the compacted vector to
   recover the surviving winner index
3. the runtime wait seam narrowed from one mixed `src/runtime/wait.rs` file
   into the focused `src/runtime/wait/poll.rs` and
   `src/runtime/wait/competition.rs` siblings, while the gateway unit suite
   moved into the folder-first
   `tests/unit/runtime/gateway/{event_based,parallel}.rs` layout so the test
   harness and modularity gate stay green together
4. the ignored local probe for this hotspot measured
   `linear_ms=434.304` versus `indexed_ms=178.365` over
   `waits=64`, `unrelated_tokens=10000`, and `iterations=128`
5. checkpoint ownership, event-competition checkpoint shape, DMN scope, and
   broader BPMN parity widening all remained unchanged during this slice

## 9. Follow-up After Boundary Wait Token Retention Slice

The next adjacent wait-path hotspot is now also closed without changing the
single-writer execution model.

What landed:

1. interrupting boundary wait settlement now resolves the winning token
   directly from parallel multi-instance owner state instead of first
   materializing a temporary `Vec<u64>` of active iteration token ids
2. the surviving timeout-path token now stays on one retention pass, so the
   runtime no longer compacts `active_tokens` and then rescans the compacted
   vector to recover the winner index
3. the internal repeat/runtime ownership seam now carries the more precise
   `parallel_multi_instance_min_token_id(...)` helper instead of the older
   broader token-id vector helper
4. the parallel multi-instance interrupting-boundary regression now also
   asserts that the surviving token keeps the smallest original token id,
   which preserves the existing bounded winner-selection rule
5. the ignored local probe for this hotspot measured
   `linear_ms=29.856` versus `indexed_ms=13.802` over
   `boundary_tokens=256`, `unrelated_tokens=10000`, and `iterations=128`
6. checkpoint ownership, multi-instance checkpoint shape, DMN scope, and
   broader BPMN parity widening all remained unchanged during this slice

## 10. Follow-up After DMN Offset Datetime Predicate Slice

The next semantics-local widening after the landed wait-path performance work
is now also closed in the DMN layer.

What landed:

1. the bounded DMN subset now accepts ISO local and RFC3339 offset-aware
   `date and time(...)` literals for comparisons and ranges, so timezone-aware
   datetime predicates no longer fall into the deferred-expression bucket
2. the evaluator compares same-kind local datetimes and same-kind
   offset-aware datetimes deterministically, while mixed local-vs-offset
   coercion still remains explicitly deferred instead of inventing broader
   FEEL temporal conversion rules
3. the LLM-facing DMN linter guidance now points remaining unsupported unary
   examples at `duration(...)`, which keeps the repair path concrete without
   misreporting timezone-aware datetime syntax as invalid
4. the runtime business-rule path now includes a focused regression proving
   one registered offset-aware datetime decision executes locally inside the
   engine and writes the expected output variables
5. the adjacent `tests/unit/dmn/datetime/` and
   `tests/unit/runtime/linear/` suites were split into folder-first seams
   during the same slice so the crate test-policy harness stayed green and no
   touched-scope structure debt was left behind
6. broader FEEL durations, custom functions, and the remaining BPMN/DMN
   parity backlog still remain outside this bounded slice

## 11. Follow-up After Transaction Compensation Slice

The next transaction-local parity widening is now also closed without changing
the single-writer checkpoint design.

What landed:

1. transaction cancel routing now restores the transaction-local variable
   snapshot first, then runs one compensation queue for completed compensable
   activities in reverse completion order, and only after that queue drains
   does the runtime route through the parent cancel boundary
2. the supported BPMN shape remains intentionally narrow:
   one compensation boundary attached to one direct host-blocking activity,
   one association to one detached `isForCompensation="true"` handler, and no
   routing, repeat, or multi-instance semantics on either side of that
   compensation binding
3. compensation handler outputs are intentionally not merged into workflow
   variables, so the rollback snapshot remains the stable state seen by later
   routing and by checkpoint persistence
4. parser validation and LLM-facing BPMN lint now surface concrete repair
   guidance for bounded compensation shape errors such as missing handler
   markers, missing associations, and unsupported routing through
   compensation-only nodes
5. the runtime transaction seam and parser compensation validator were both
   split into smaller owners during the same slice so modularity and clippy
   closure moved forward with the semantics work instead of becoming new debt
6. broader compensation surfaces such as throw compensation events,
   compensation event subprocesses, and default compensation still remain
   outside this bounded slice

## 12. Follow-up After DMN Multi-Decision Source Slice

The next parser-local parity widening is now also closed without changing the
single-writer checkpoint design.

What landed:

1. one DMN source may now contain multiple bounded `<decision>` elements, with
   each decision still constrained to exactly one `decisionTable` and the
   existing `UNIQUE`/`COLLECT` evaluator subset
2. the public/internal DMN parser now exposes one plural parse path for that
   bounded source shape, while the exact-one `parse_dmn_decision(...)`
   wrapper still rejects multi-decision files instead of silently picking one
   decision
3. parser-owned BPMN bundle snapshots now register every parsed decision from
   an attached DMN source into the package registry, so local business-rule
   lookup stays deterministic without inventing a broader nested registry
4. the LLM-facing DMN linter now accepts valid multi-decision sources and
   keeps repair guidance focused on the still-deferred cases:
   zero decisions, multiple `decisionTable` blocks within one decision,
   unsupported FEEL, and unsupported hit policies
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 13. Follow-up After DMN Document Snapshot Slice

The next DMN placeholder slice closed without changing the single-writer
checkpoint design. A later bounded runtime slice now narrows the direct
relation executable subset.

What landed:

1. the crate now exposes one non-executable DMN document snapshot surface that
   records root metadata, DMN model namespace/version hints, and per-decision
   decision-table counts for later DMN and adapter work
2. the LLM-facing DMN linter now carries that snapshot context into
   unsupported executable-shape reports, so a versioned or namespaced DMN file
   can explain which document root and model-version hint were discovered
   before the bounded executable parser stopped
3. focused fixtures now prove both halves of the current contract:
   a syntactically valid versioned DMN document that still lacks a
   `decisionTable`, and a namespaced versioned document that does remain
   executable inside the existing bounded decision-table subset
4. the snapshot owner was split into the folder-first
   `src/dmn/snapshot/{scan,xml}.rs` seam during the same slice so modularity
   closure landed together with the new placeholder surface
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 14. Follow-up After DMN Unsupported Construct Classification Slice

The next DMN placeholder slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN snapshot surface now records top-level
   `decisionService` counts and per-decision direct `literalExpression`
   counts, so later adapter and lint work can distinguish those unsupported
   constructs from generic missing-decision or missing-table shapes
2. the LLM-facing DMN linter now emits the construct-specific codes
   `dmn.unsupported_decision_service` and
   `dmn.unsupported_literal_expression_decision`, with repair guidance that
   explicitly tells callers not to fabricate decision-table logic from either
   metadata-only `decisionService` contracts or direct FEEL
   `literalExpression` bodies
3. focused versioned fixtures now prove the new placeholder surface on both
   sides:
   one 20180521 namespaced DMN document containing only `decisionService`,
   and one 20191111 namespaced DMN document containing one direct
   `literalExpression` decision, while both remain non-executable inside the
   bounded evaluator subset
4. the snapshot owner widened into the folder-first
   `src/dmn/snapshot/{scan,root,state,xml}.rs` seam during the same slice,
   and the DMN document-error lint mapping was split into smaller helpers, so
   modularity and strict clippy closure both landed together with the new
   placeholder diagnostics
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 14.1 Follow-up After DMN Literal Expression Runtime Slice

The earlier placeholder-only `literalExpression` note is now narrowed.

What landed:

1. the bounded DMN parser can materialize one direct decision-owned
   `<literalExpression><text>` body without requiring a decision table
2. the bounded evaluator can execute supported direct literal-expression text:
   one supported literal, one variable path, or one whitespace-delimited numeric
   `path +/- number` operation
3. the DMN linter now accepts that direct-literal subset and reports
   `dmn.unsupported_literal_expression_subset` only for broader FEEL text, with
   repair guidance that keeps LLM fixes from fabricating decision-table rules
4. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model remained unchanged

## 14.2 Follow-up After DMN List Expression Runtime Slice

The earlier placeholder-only direct `list` note is now narrowed.

What landed:

1. the bounded DMN parser can materialize one direct decision-owned `<list>`
   without requiring a decision table when every direct child is a bounded
   `<literalExpression>` item
2. the bounded evaluator can execute each list item through the existing
   literal-expression subset and merge output as
   `{ "<decision_id>": [<values>...] }`
3. the DMN linter now accepts that direct-list subset and reports
   `dmn.unsupported_list_expression_subset` for unsupported item text or
   `dmn.unsupported_list_child` for non-literal direct children
4. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model remained unchanged

## 14.3 Follow-up After DMN Context Expression Runtime Slice

The earlier placeholder-only direct `context` note is now narrowed.

What landed:

1. the bounded DMN parser can materialize one direct decision-owned
   `<context>` without requiring a decision table when every direct
   `<contextEntry>` contains optional variable metadata and one bounded
   `<literalExpression>` body
2. the bounded evaluator executes entries in source order, makes named entries
   visible to later entries, and returns one final unnamed entry as
   `{ "<decision_id>": <value> }`
3. the DMN linter now accepts that direct-context subset and reports
   `dmn.unsupported_context_expression_subset` for unsupported entry text or
   `dmn.unsupported_context_child` for children outside the bounded
   context-entry shape
4. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model remained unchanged

## 15. Follow-up After DMN Context and Invocation Classification Slice

The next DMN placeholder slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records direct `context`
   counts and direct `invocation` counts, so later adapter and lint work can
   distinguish those decision-logic shapes from the generic missing-table
   fallback; direct `context` execution is now narrowed by the bounded runtime
   follow-up above
2. the LLM-facing DMN linter now emits the construct-specific codes
   `dmn.unsupported_context_decision` and
   `dmn.unsupported_invocation_decision`, with repair guidance that
   explicitly tells callers not to flatten context entries or fabricate
   invocation rewrites into guessed decision-table rules
3. focused 20191111 namespaced fixtures prove both construct surfaces: one
   direct bounded `context` decision and one direct `invocation` decision,
   while invocation remains non-executable inside the bounded evaluator subset
4. the slice stayed inside the existing folder-first snapshot and lint seams,
   and after the earlier snapshot/lint owner refactors, no new touched-scope
   modularity or strict clippy debt surfaced during full validation
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 16. Follow-up After DMN Relation Classification Slice

The next DMN placeholder slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records direct `relation`
   counts, so later adapter and lint work can distinguish that unsupported
   decision-logic shape from the generic missing-table fallback
2. the LLM-facing DMN linter emitted the construct-specific code
   `dmn.unsupported_relation_decision`, with repair guidance that explicitly
   told callers not to flatten relation rows into guessed decision-table
   rules before the bounded direct-relation runtime subset existed
3. one focused 20191111 namespaced fixture proved the placeholder surface for
   one direct `relation` decision while keeping it non-executable inside that
   slice's bounded evaluator subset
4. full validation surfaced one immediate touched-scope project-harness debt in
   the DMN lint suite, and it was closed in the same slice by splitting the
   tests into the folder-first `tests/unit/lint/dmn/{mod,core,constructs}.rs`
   seam
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 17. Follow-up After DMN Function and List Classification Slice

The next DMN placeholder slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records direct
   `functionDefinition` counts and direct `list` counts, so later adapter and
   lint work can distinguish the remaining direct `decisionLogic` expression
   shapes allowed by the DMN 1.3 schema from the generic missing-table
   fallback
2. the LLM-facing DMN linter now emits the construct-specific codes
   `dmn.unsupported_function_definition_decision` and
   `dmn.unsupported_list_decision`, with repair guidance that explicitly
   tells callers not to inline function bodies or flatten list items into
   guessed decision-table rules
3. focused 20191111 namespaced fixtures now prove both placeholder surfaces:
   one direct `functionDefinition` decision and one direct `list` decision,
   while both remain non-executable inside the bounded evaluator subset
4. after the earlier folder-first DMN lint split, full validation did not
   surface any new touched-scope testing, modularity, or strict clippy debt
   in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 18. Follow-up After DMN Document Root Validation Slice

The next DMN document-validation slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the executable DMN parser now rejects non-`definitions` roots and
   missing or unsupported DMN model namespaces before decision parsing
   continues, so invalid documents now fail at the document seam instead of
   falling through into misleading placeholder diagnostics
2. the LLM-facing DMN linter now emits the document-level issue codes
   `dmn.invalid_root_element`,
   `dmn.missing_model_namespace`, and
   `dmn.unsupported_model_namespace`, which is materially safer for repair
   flows than the earlier generic decision-level fallback
3. the required root attributes `definitions@name` and
   `definitions@namespace` are now enforced through the existing
   missing-attribute path, so the bounded schema/version surface is stricter
   without widening into full XSD validation
4. full validation surfaced touched-scope modularity and strict clippy debt
   inside the DMN parse XML seam, and that debt was closed in the same slice
   by splitting the owner into the folder-first
   `src/dmn/parse/xml/{api,decode,end,root,start}.rs` seam plus
   `src/dmn/parse/driver.rs`
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 19. Follow-up After DMN Import Validation Slice

The next DMN document-validation slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level `import`
   counts, so later adapter and lint work can distinguish cross-document DMN
   dependency declarations from ordinary local bounded decision-table files
2. the executable DMN parser now rejects top-level `<import>` declarations
   before decision-table execution begins, so imported DMN sources no longer
   look locally executable just because the engine ignored that surface
3. the LLM-facing DMN linter now emits the document-level issue code
   `dmn.unsupported_import`, with repair guidance that explicitly tells
   callers not to delete imports blindly just to force local parsing
4. one focused 20191111 namespaced fixture now proves the import surface
   without widening into DMN import resolution, item-definition execution, or
   cross-document decision execution, and full validation did not surface any
   new touched-scope testing, modularity, or strict clippy debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 20. Follow-up After DMN Requirement Edge Classification Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records direct
   `informationRequirement`,
   `knowledgeRequirement`, and
   `authorityRequirement` counts, so later adapter and lint work can
   distinguish dependency-edge-only decisions from local decision-table
   shapes
2. the LLM-facing DMN linter now emits the construct-specific codes
   `dmn.unsupported_information_requirement_decision`,
   `dmn.unsupported_knowledge_requirement_decision`, and
   `dmn.unsupported_authority_requirement_decision`, with guidance that
   explicitly tells callers not to fabricate local decision-table rules from
   requirement edges alone
3. focused namespaced fixtures now prove all three requirement-edge surfaces
   without widening into DMN dependency resolution, DRD execution, or
   evaluator semantics
4. full validation first surfaced one touched-scope DMN snapshot
   test-structure debt and one strict clippy long-function debt in the same
   seam; both were closed immediately by splitting
   `tests/unit/dmn/snapshot/` folder-first and by extracting
   `TempDecisionSnapshot::from_event`,
   `start_decision`,
   `track_root_construct`, and
   `track_decision_construct` from `src/dmn/snapshot/state.rs`
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 21. Follow-up After DMN Requirement Target Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records nested
   `requiredInput`,
   `requiredDecision`,
   `requiredKnowledge`, and
   `requiredAuthority` counts under the existing requirement-edge seam, so
   later adapter and lint work can distinguish which upstream target shape a
   dependency-only decision actually references
2. the LLM-facing DMN linter now keeps the existing requirement-edge issue
   codes but emits target-aware guidance and evidence when one explicit
   target shape is present, which is materially safer for repair flows than
   broader edge-only wording
3. focused namespaced fixtures now prove required input, required decision,
   required knowledge, and required authority surfaces without widening into
   DMN dependency resolution, DRD execution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra target coverage, and full validation did not
   surface any new touched-scope testing, modularity, or strict clippy debt
   in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 22. Follow-up After DMN Root Artifact Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level
   `inputData`,
   `knowledgeSource`, and
   `businessKnowledgeModel` counts alongside the earlier `import` and
   `decisionService` counts, so later adapter and lint work can distinguish
   metadata-only DRD documents from ordinary empty definitions files
2. the LLM-facing DMN linter now emits document-level artifact-aware
   guidance when one explicit top-level metadata shape is present without any
   executable `<decision>`, which is materially safer for repair flows than
   the earlier generic missing-decision wording
3. focused namespaced fixtures now prove input-data-only,
   knowledge-source-only, business-knowledge-model-only, and generic
   empty-definitions surfaces without widening into DRD execution,
   dependency resolution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra root-artifact coverage, and full validation did
   not surface any new touched-scope testing, modularity, or strict clippy
   debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 23. Follow-up After DMN Item Definition Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level
   `itemDefinition` counts alongside the earlier `import`, `inputData`,
   `knowledgeSource`, `businessKnowledgeModel`, and `decisionService`
   counts, so later adapter and lint work can distinguish type-model-only
   DMN documents from other metadata-only root shapes
2. the LLM-facing DMN linter now emits document-level item-definition-aware
   guidance when one explicit top-level `itemDefinition` shape is present
   without any executable `<decision>`, which is materially safer for repair
   flows than the earlier generic missing-decision wording
3. one focused namespaced fixture now proves item-definition-only surfaces
   without widening into item-definition resolution, artifact execution,
   dependency resolution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra item-definition coverage, and full validation did
   not surface any new touched-scope testing, modularity, or strict clippy
   debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 24. Follow-up After DMN Business Context Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level
   `organizationUnit` and `performanceIndicator` counts alongside the earlier
   `import`, `itemDefinition`, `inputData`, `knowledgeSource`,
   `businessKnowledgeModel`, and `decisionService` counts, so later adapter
   and lint work can distinguish business-context-only DMN documents from
   other metadata-only root shapes
2. the LLM-facing DMN linter now emits document-level business-context-aware
   guidance when one explicit top-level governance shape is present without
   any executable `<decision>`, which is materially safer for repair flows
   than the earlier generic missing-decision wording
3. focused namespaced fixtures now prove organization-unit-only and
   performance-indicator-only surfaces without widening into
   business-context resolution, owner/ownership reference parsing, artifact
   execution, dependency resolution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra business-context coverage, and full validation did
   not surface any new touched-scope testing, modularity, or strict clippy
   debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 25. Follow-up After DMN Text Annotation Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level
   `textAnnotation` counts alongside the earlier `import`, `itemDefinition`,
   `inputData`, `knowledgeSource`, `businessKnowledgeModel`,
   `decisionService`, `organizationUnit`, and `performanceIndicator`
   counts, so later adapter and lint work can distinguish annotation-only
   DMN documents from other metadata-only root shapes
2. the LLM-facing DMN linter now emits document-level text-annotation-aware
   guidance when one explicit top-level annotation shape is present without
   any executable `<decision>`, which is materially safer for repair flows
   than the earlier generic missing-decision wording
3. one focused namespaced fixture now proves text-annotation-only surfaces
   without widening into annotation resolution, `association`,
   `elementCollection`, DMNDI relationships, dependency resolution, or
   evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra text-annotation coverage, and full validation did
   not surface any new touched-scope testing, modularity, or strict clippy
   debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 26. Follow-up After DMN Association and Element Collection Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level `association`
   and `elementCollection` counts alongside the earlier `import`,
   `itemDefinition`, `inputData`, `knowledgeSource`,
   `businessKnowledgeModel`, `decisionService`, `organizationUnit`,
   `performanceIndicator`, and `textAnnotation` counts, so later adapter and
   lint work can distinguish document-structure-only DMN files from other
   metadata-only root shapes
2. the LLM-facing DMN linter now emits document-level association-aware and
   element-collection-aware guidance when one explicit top-level
   document-structure shape is present without any executable `<decision>`,
   which is materially safer for repair flows than the earlier generic
   missing-decision wording
3. focused namespaced fixtures now prove association-only and
   element-collection-only surfaces without widening into association
   resolution, element-collection membership parsing, DMNDI relationships,
   dependency resolution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra document-structure coverage, and full validation
   did not surface any new touched-scope testing, modularity, or strict
   clippy debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 27. Follow-up After DMN DMNDI Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level
   `dmndi:DMNDI` counts alongside the earlier `import`, `itemDefinition`,
   `inputData`, `knowledgeSource`, `businessKnowledgeModel`,
   `decisionService`, `organizationUnit`, `performanceIndicator`,
   `textAnnotation`, `association`, and `elementCollection` counts, so
   later adapter and lint work can distinguish diagram-only DMN files from
   other metadata-only root shapes
2. the LLM-facing DMN linter now emits document-level DMNDI-aware guidance
   when one explicit top-level diagram-interchange block is present without
   any executable `<decision>`, which is materially safer for repair flows
   than the earlier generic missing-decision wording
3. one focused namespaced fixture now proves DMNDI-only surfaces without
   widening into DMNDI relationship parsing, `DMNDiagram` / `DMNShape` /
   `DMNEdge` resolution, dependency resolution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra diagram-interchange coverage, and full validation
   did not surface any new touched-scope testing, modularity, or strict
   clippy debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 28. Follow-up After DMN Group Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN root snapshot now records top-level `group`
   counts alongside the earlier `import`, `itemDefinition`, `inputData`,
   `knowledgeSource`, `businessKnowledgeModel`, `decisionService`,
   `organizationUnit`, `performanceIndicator`, `textAnnotation`,
   `association`, `elementCollection`, and `dmndi:DMNDI` counts, so later
   adapter and lint work can distinguish group-only DMN files from other
   metadata-only root shapes
2. the LLM-facing DMN linter now emits document-level group-aware guidance
   when one explicit top-level `group` artifact is present without any
   executable `<decision>`, which is materially safer for repair flows than
   the earlier generic missing-decision wording
3. one focused namespaced fixture now proves group-only surfaces without
   widening into group resolution, DMNDI relationships, dependency
   resolution, or evaluator semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra group coverage, and full validation did not
   surface any new touched-scope testing, modularity, or strict clippy debt
   in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 29. Follow-up After DMN Allowed Answers Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records direct
   `allowedAnswers` counts alongside the earlier decision-table,
   requirement, and expression counts, so later adapter and lint work can
   distinguish metadata-only decision output hints from executable
   decision-table logic
2. the LLM-facing DMN linter now emits decision-level allowed-answers-aware
   guidance when one explicit `allowedAnswers` metadata block is present
   without any executable local `<decisionTable>`, which is materially safer
   for repair flows than the earlier generic missing-decision-table wording
3. one focused namespaced fixture now proves `allowedAnswers`-only decision
   surfaces without widening into FEEL evaluation, output coercion,
   decision-table metadata support, or broader decision-metadata semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra decision-metadata coverage, and full validation
   did not surface any new touched-scope testing, modularity, or strict
   clippy debt in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 30. Follow-up After DMN Decision Governance Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the non-executable DMN decision snapshot now records direct
   `decisionMaker` and `decisionOwner` counts alongside the earlier
   decision-table, requirement, expression, and `allowedAnswers` counts, so
   later adapter and lint work can distinguish governance-only decision
   metadata from executable decision-table logic
2. the LLM-facing DMN linter now emits decision-level governance-aware
   guidance when one explicit maker-only or owner-only governance block is
   present without any executable local `<decisionTable>`, which is
   materially safer for repair flows than the earlier generic
   missing-decision-table wording
3. focused namespaced fixtures now prove maker-only and owner-only decision
   surfaces without widening into identity resolution, mixed governance
   metadata classification, FEEL evaluation, or broader governance semantics
4. the earlier folder-first DMN snapshot and lint test seams gave enough
   headroom for the extra governance coverage, and full validation did not
   surface any new touched-scope testing, modularity, or strict clippy debt
   in this slice
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 31. Follow-up After DMN Mixed Decision Governance Guidance Slice

The next DMN lint-precision slice is now also closed without changing the
single-writer checkpoint design.

What landed:

1. the existing non-executable DMN decision snapshot counts for
   `decisionMaker` and `decisionOwner` are now consumed together so the
   linter can recognize mixed governance-only decision shapes instead of
   dropping them into the generic missing-decision-table fallback
2. the LLM-facing DMN linter now emits decision-level mixed-governance
   guidance when one decision carries both metadata families without any
   executable local `<decisionTable>`, which is materially safer for repair
   flows than the earlier generic missing-decision-table wording
3. one focused namespaced fixture now proves the mixed-governance decision
   surface without widening into identity resolution, FEEL evaluation, or
   broader governance semantics
4. full validation surfaced one local Valkey live-test helper port-conflict
   race in the same crate, and that touched verification debt was closed in
   the same slice without changing checkpoint semantics
5. checkpoint ownership, distributed writer ownership, and the Valkey-backed
   single-writer checkpoint model all remained unchanged during this slice

## 32. Follow-up After Frontier Snapshot Classification Performance Slice

The next bounded runtime-performance slice is now closed inside the existing
frontier snapshot model. The first dense-status cut was superseded in the same
performance lane by a stronger direct-status implementation.

What landed:

1. deterministic active-token snapshot order remains unchanged
2. `rayon` remains limited to immutable frontier inspection
3. frontier snapshot classification no longer builds an extra queued-node hash
   set for every snapshot pass
4. frontier snapshot classification also no longer allocates a per-snapshot
   dense node-status vector
5. queued-owner and terminal-node classification now directly indexes the
   borrowed immutable node runtime state for active tokens
6. direct waits and boundary-blocking waits now share one wait-role lookup,
   using a sparse map for narrow fronts and an adaptive dense vector for wider
   wait/token fronts
7. focused regression coverage proves queued boundary-blocking owner tokens
   still classify as runnable while non-queued owners remain waiting
8. the ignored local probe for this hotspot measured
   `hashset_ms=454.223`, `dense_status_ms=362.312`,
   `sparse_direct_status_ms=200.753`, and
   `adaptive_direct_status_ms=51.450` over `nodes=20000`, `tokens=10000`,
   `waits=512`, and `iterations=128`
9. the single-writer Valkey checkpoint contract and public frontier API
   remained unchanged

## 33. Follow-up After Frontier Runtime Planning Fast Path Slice

The next bounded runtime-performance slice moved below public frontier snapshot
construction and into the internal advance loop. Public planning remains
available for diagnostics and API consumers, but the runtime no longer pays
that public-shape materialization cost on every advance iteration.

What landed:

1. `advance_instance` now uses a crate-private runtime planner that collects
   runnable execution proposals directly from immutable frontier state instead
   of calling the public `plan_frontier_step` snapshot-and-batch surface
2. the runtime planner preserves the public idle outcomes for blocked host
   work, external waits, suspension, and stalled frontiers
3. the runtime batch now keeps the common no-parallel-join path as raw
   proposals, so ordinary multi-token execution does not wrap every runnable
   token into `BpmnFrontierExecutionStep::Proposal`
4. the parallel-join path still falls back to merge-aware execution steps, so
   deterministic join coalescing and token re-indexing semantics remain
   unchanged
5. focused frontier tests still pass across public planning, idle outcomes,
   queued boundary-owner classification, and parallel-join merge execution
6. the ignored local runtime planning probe measured
   `public_snapshot_ms=128.019`, `direct_wrapped_steps_ms=97.623`, and
   `runtime_fast_path_ms=53.191` over `nodes=20000`, `tokens=10000`, and
   `iterations=128`
7. the single-writer Valkey checkpoint contract, public frontier API, and
   external BPMN behavior remained unchanged

## 34. Follow-up After Token ID Allocation Fan-out Performance Slice

The next bounded runtime-performance slice moved into mutable fan-out token
creation. Public runtime state and checkpoint shape remain unchanged; the
optimization is a local allocation strategy inside one fan-out operation.

What landed:

1. single-token routing still uses the existing `next_token_id` behavior
2. parallel gateway, inclusive gateway, event-based gateway, and parallel
   multi-instance fan-out now create a local token-id allocator after the
   existing active and pending token sets are scanned once
3. fan-out token order and deterministic active frontier order remain
   unchanged
4. the allocator is crate-private runtime state and is not serialized into
   checkpoints
5. focused gateway and multi-instance runtime tests still pass after the
   allocator-backed fan-out change
6. the ignored local token allocation probe measured
   `repeated_scan_ms=2729.932` and `allocator_ms=1.391` over
   `initial_tokens=8000`, `pending_tokens=512`, `pushed_tokens=2048`, and
   `iterations=16`
7. the single-writer Valkey checkpoint contract, public frontier API, and
   external BPMN behavior remained unchanged

## 35. Follow-up After Event Competition Retain Performance Slice

The next bounded wait-runtime performance slice tightened event-based gateway
resume. The larger wait-node membership optimization was already in place, so
this slice focused only on removing an avoidable second active-token scan.

What landed:

1. event competition winner validation still checks that the selected wait
   belongs to the event-based gateway owner
2. the winner token is now retained during the same active-token pass that
   removes competing wait tokens
3. exactly one token for the winning wait node is retained, while unrelated
   active tokens keep deterministic order
4. focused event-based gateway runtime tests still pass
5. the ignored local event competition probe measured
   `linear_ms=455.419`, `indexed_ms=187.858`, and
   `fused_indexed_ms=187.094` over `waits=64`,
   `unrelated_tokens=10000`, and `iterations=128`
6. the small delta confirms that the remaining cost center is broader wait
   resolution and process lookup, not winner-token pre-scan alone
7. the single-writer Valkey checkpoint contract, public frontier API, and
   external BPMN behavior remained unchanged

## 36. Follow-up After Wait Process Resolution Performance Slice

The next bounded wait-runtime performance slice removed package-wide process
lookup from event-poll apply when the cached process index is still valid.
Fallback lookup remains in place for stale checkpoints.

What landed:

1. current-frame waits now resolve the owning process through
   `BpmnInstanceState.process_index` first
2. parent-frame waits now resolve through `CallActivityFrame.process_index`
   first
3. stale or missing indexes still fall back to package `process_id` lookup, so
   older checkpoints remain recoverable
4. focused wait, event-based gateway, and call-activity boundary/runtime tests
   still pass
5. the ignored local wait process lookup probe measured
   `linear_ms=59506.419` and `indexed_ms=6.041` over `processes=20000` and
   `iterations=200000`
6. the single-writer Valkey checkpoint contract, public frontier API, and
   external BPMN behavior remained unchanged

## 37. Follow-up After Frontier Proposal Index Fast Path Slice

The next bounded frontier-runtime performance slice used data that the runtime
already carries. Each execution proposal includes the token index observed
during frontier planning; the runtime now validates that index before falling
back to token-id lookup.

What landed:

1. frontier proposal execution first checks whether `proposal.token_index`
   still points at the same token id, node index, and incoming edge
2. stale proposal indexes still fall back to the existing token-id lookup, so
   token removal or re-indexing cases remain safe
3. focused frontier tests still pass, including the parallel-join cases that
   force re-indexing after token removal
4. the ignored local frontier token lookup probe measured
   `linear_ms=780.653`, `batch_lookup_ms=111.312`, and
   `proposal_index_ms=0.672` over `tokens=10000`,
   `lookups_per_batch=512`, and `iterations=64`
5. the single-writer Valkey checkpoint contract, public frontier API, and
   external BPMN behavior remained unchanged

## 38. Follow-up After DMN Relation Expression Runtime Slice

The next DMN boxed-expression runtime slice closed one more direct
decision-owned expression shape without changing checkpoint ownership.

What landed:

1. one direct decision-owned `<relation>` can now parse and execute when it
   contains direct columns and rows with one bounded `<literalExpression>` cell
   per column
2. relation runtime output remains deterministic and object-shaped as
   `{ "<decision_id>": [{ "<column_key>": <cell_value>, ... }, ...] }`
3. the DMN linter now accepts that direct-relation subset and reports
   `dmn.unsupported_relation_expression_subset` for unsupported cell text or
   `dmn.unsupported_relation_child` for children outside the bounded
   column/row shape
4. nested relations, broader boxed cell expressions, imports, DRD dependency
   execution, full schema validation, and broader FEEL semantics remain
   deferred
5. the parser refactor closed immediate touched-scope clippy debt without
   adding lint suppression, and the single-writer Valkey checkpoint contract
   remained unchanged

## 39. Follow-up After DMN Invocation Snapshot Evidence Slice

The next DMN alignment slice widened non-executable invocation evidence without
changing runtime semantics or checkpoint ownership.

What landed:

1. direct decision-owned `<invocation>` remains non-executable inside the
   bounded evaluator
2. the non-executable decision snapshot now preserves the direct invoked
   literal-expression text and each direct binding's parameter plus argument
   literal-expression text
3. `dmn.unsupported_invocation_decision` evidence now carries that invocation
   structure for `qianji lint --dmn` and future adapter flows
4. called-function resolution, business-knowledge-model execution, binding
   evaluation, imports, DRD dependency execution, full schema validation, and
   broader FEEL semantics remain deferred
5. the single-writer Valkey checkpoint contract and runtime execution behavior
   remained unchanged

## 40. Follow-up After DMN Function Definition Snapshot Evidence Slice

The next DMN alignment slice widened non-executable function-definition
evidence without changing runtime semantics or checkpoint ownership.

What landed:

1. direct decision-owned `<functionDefinition>` remains non-executable inside
   the bounded evaluator
2. the non-executable decision snapshot now preserves the direct function id,
   function kind, formal-parameter metadata, and body literal-expression text
3. `dmn.unsupported_function_definition_decision` evidence now carries that
   function structure for `qianji lint --dmn` and future adapter flows
4. function body evaluation, business-knowledge-model execution, imports, DRD
   dependency execution, full schema validation, and broader FEEL semantics
   remain deferred
5. the single-writer Valkey checkpoint contract and runtime execution behavior
   remained unchanged

## 41. Follow-up After DMN Business-Knowledge-Model Body Snapshot Evidence Slice

The next DMN alignment slice widened non-executable BKM evidence without
changing runtime semantics or checkpoint ownership.

What landed:

1. top-level `<businessKnowledgeModel>` remains non-executable inside the
   bounded evaluator
2. the non-executable document root snapshot now preserves the direct BKM body
   `literalExpression` id, optional typeRef, and text payload
3. `dmn.unsupported_business_knowledge_model_artifact` evidence now carries
   that body structure for `qianji lint --dmn` and future adapter flows
4. BKM body evaluation, invocation binding, imports, DRD dependency execution,
   full schema validation, and broader FEEL semantics remain deferred
5. the single-writer Valkey checkpoint contract and runtime execution behavior
   remained unchanged

## 42. Follow-up After DMN Decision-Service Reference Snapshot Evidence Slice

The next DMN alignment slice widened non-executable decision-service evidence
without changing runtime semantics or checkpoint ownership.

What landed:

1. top-level `<decisionService>` remains non-executable inside the bounded
   evaluator
2. the non-executable document root snapshot now preserves direct
   `outputDecision`, `encapsulatedDecision`, `inputDecision`, and `inputData`
   href placeholders
3. `dmn.unsupported_decision_service` evidence now carries those references for
   `qianji lint --dmn` and future adapter flows
4. decision-service reference resolution, output-decision execution, imports,
   DRD dependency execution, full schema validation, and broader FEEL
   semantics remain deferred
5. the single-writer Valkey checkpoint contract and runtime execution behavior
   remained unchanged

## 43. Follow-up After DMN Requirement Reference Snapshot Evidence Slice

The next DMN alignment slice widened non-executable requirement-edge evidence
without changing runtime semantics or checkpoint ownership.

What landed:

1. decision-owned `informationRequirement`, `knowledgeRequirement`, and
   `authorityRequirement` remain non-executable dependency edges inside the
   bounded evaluator
2. the non-executable decision snapshot now preserves direct target hrefs with
   parent requirement kind and target reference kind
3. unsupported requirement-decision lint evidence now carries those hrefs for
   `qianji lint --dmn` and future adapter flows
4. DRD dependency execution, href resolution, imports, full schema validation,
   and broader FEEL semantics remain deferred
5. the single-writer Valkey checkpoint contract and runtime execution behavior
   remained unchanged
