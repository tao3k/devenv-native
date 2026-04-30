# BPMN Alignment Index

This module tracks how `qianji-bpmn-engine` aligns with the official
[BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

The current engine is intentionally bounded. Alignment means:

- the parser accepts one explicit BPMN shape
- the runtime executes that same shape deterministically
- the lint surface explains unsupported shapes in an LLM-friendly way

The source-backed clause registry for these notes lives in
[BPMN Official Source Map](spec-source-map.md).

## Current Modules

- [Official Source Map](spec-source-map.md)
- [Collaboration, Lanes, and Data](collaboration-lanes-and-data.md)
- [Events and Boundaries](events-and-boundaries.md)
- [Full Conformance Coverage](full-conformance-coverage.md)
- [Gateways and Concurrency](gateways-and-concurrency.md)
- [Human Interaction Host-Loop Audit](human-interaction-host-loop-audit.md)
- [Human Interaction Milestone Plan](human-interaction-milestone-plan.md)
- [Host Request ABI Ledger](host-request-abi-ledger.md)
- [Loops and Multi-Instance](loops-and-multi-instance.md)
- [Subprocesses, Transactions, and Compensation](subprocesses-transactions-and-compensation.md)
- [Tasks and Host Dispatch](tasks-and-host-dispatch.md)

## Current Package Boundary

The current package owns bounded support for:

- linear flows and bounded gateway routing
- a Rust-owned BPMN conformance registry that keeps the full coverage matrix
  machine-checkable
- bounded start/intermediate waits and boundary events
- bounded loop and multi-instance task execution
- bounded host-dispatched task families including `sendTask` and `scriptTask`
- bounded subprocess, transaction, and same-package call-activity slices
- bounded transaction-owned compensation slices
- bounded interrupting event-subprocess execution for one trigger shape
- bounded task-local Data/IO through native `ioSpecification`,
  `dataInputAssociation`, and `dataOutputAssociation` mappings
- bounded process-level data-object copy-in/copy-out through standard task
  data associations
- explicit data-store binding diagnostics that keep `dataStoreReference`
  persistence out of executable task IO until a storage policy exists
- a Rust-owned callable registry for process/global-task metadata, callable
  IO metadata, and existing same-package process-target callActivity bindings
- explicit global-task callActivity diagnostics that keep top-level
  global-task definitions metadata-only until a bounded execution policy exists
- explicit operation-binding diagnostics that keep task-level `operationRef`
  metadata from implying interface-operation invocation
- a Rust-owned collaboration host envelope for collaboration shells,
  participants, message-flow intent, correlation properties, correlation
  keys, and process correlation subscriptions
- a native BPMN compatibility proof that representative standard XML with BPMN
  DI and task IO parses, lints, and runs without custom XML namespaces or
  custom moddle descriptors
- non-executable BPMN document snapshots for collaboration, partner,
  participant, choreography, artifact, lane, data-store, import,
  extension, relationship, BPMN DI, conversation, global task, process
  callable, callable IO, IO-set, data-state, data-association expression,
  resource-role, flow-element, catalog, and category metadata

The current package still defers:

- collaboration and lane semantics
- full BPMN data-store execution and broader IO execution coverage
- complex-gateway activation and unstructured synchronization semantics
- unbounded event families and unsupported event-subprocess shapes
- broader FEEL or script-backed flow semantics

Deferred collaboration, choreography, artifact, lane, data-store, complex
gateway,
import, extension, relationship, BPMN DI, global task, process callable,
callable IO, IO-set, data-state, data-association expression, resource-role,
flow-element, category, and unsupported IO surfaces are reported by the linter
with explicit repair guidance instead of being treated as executable runtime
semantics. Those lint reports also include bounded snapshot-derived evidence
for the deferred family, such as
participant/message-flow counts, conversation node/link/association counts,
partner/entity/role counts, participant interface/endpoint/multiplicity
metadata, choreography activity counts, artifact association/group and
text-annotation counts, lane flow-node refs, data-object and data-association
references, direct `dataState` metadata on standard BPMN data owners,
data-store-reference binding evidence, IO-set reference metadata,
data-association `transformation` and `assignment` payloads,
process support/property/correlation-subscription metadata,
process/global-task resource-role metadata, direct callable IO binding
metadata, global-task IO specification metadata, direct flow-element
auditing/monitoring/category metadata, or diagram element counts.

The collaboration host envelope is also metadata-only. It lets hosts inspect
participant, message-flow, and correlation intent from the parsed package, but
it does not execute pool routing, participant dispatch, endpoint invocation,
message-flow routing, or BPMN correlation matching. Runtime wait
`deduplication_key` remains a host event de-duplication hint derived from
explicit event references, not a BPMN correlation key.
