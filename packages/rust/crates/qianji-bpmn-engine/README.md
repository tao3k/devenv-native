# qianji-bpmn-engine

Bounded BPMN and DMN workflow engine ownership for Qianji.

## Responsibility

`qianji-bpmn-engine` owns the engine-side core for:

- BPMN parsing, bounded execution, and host-blocked workflow runtime
- DMN parsing, bounded evaluation, and LLM-friendly lint diagnostics
- bounded ISO date, datetime, time, and signed day-time or year-month
  duration decision predicates
- checkpoint codecs plus distributed Valkey-backed checkpoint ownership
- bounded `exclusiveGateway` condition routing with simple boolean-path or
  numeric-comparison `sequenceFlow` conditions plus one optional `default`
  branch
- bounded structured `inclusiveGateway` split/join routing with the same
  bounded condition subset plus one matching linear join fragment
- bounded transaction cancel and error routing, including one explicit
  transaction-cancel compensation subset with reverse completion replay plus
  one bounded throw-compensation end-event subset that may stay synchronous
  or set `waitForCompletion="false"` with either explicit `activityRef`
  targeting or bounded default replay plus one synchronous or asynchronous
  throw-compensation intermediate-event subset with either explicit
  `activityRef` targeting or bounded default replay
- bounded top-level `errorEventDefinition` end termination for one executable
  process, producing one terminal failed instance outcome with preserved
  variables
- bounded interrupting timer, message, or signal `boundaryEvent` execution
  on one host-blocking task, where the winning boundary event cancels the
  blocked task and routes the existing token down the boundary path
- bounded interrupting timer, message, or signal `boundaryEvent` execution
  on one embedded subprocess owner, where the parent boundary stays armed
  while the child shell runs and the winning boundary cancels that child
  shell before restoring the parent token onto the boundary path
- bounded interrupting timer, message, or signal `boundaryEvent` execution
  on one same-package `callActivity` owner, where the parent boundary stays
  armed while the called child process runs and the winning boundary cancels
  that child process before restoring the parent token onto the boundary
  path
- bounded interrupting timer, message, or signal `boundaryEvent` execution
  on one bounded transaction shell owner, where the parent boundary stays
  armed while the transaction child shell runs and the winning boundary
  cancels that child shell before restoring the parent token onto the
  boundary path; the bounded mixed-owner subset may pair that same
  timer/message/signal boundary with one interrupting cancel boundary, with
  one or more interrupting error boundaries, or with one interrupting cancel
  boundary plus one or more interrupting error boundaries on the same owner,
  while still permitting only one interrupting timer/message/signal boundary
  and one interrupting cancel boundary on that same transaction shell
- bounded non-interrupting timer, message, or signal `boundaryEvent`
  execution on one non-repeating or bounded standard-loop, sequential
  multi-instance, or parallel multi-instance host-blocking task, where
  boundary firing opens one concurrent path while original task work stays
  active
- bounded embedded-subprocess and same-package `callActivity` error routing
  through one or more matching parent interrupting error boundaries,
  including one optional catch-all boundary on the same owner; on one
  embedded subprocess owner, the bounded mixed-owner subset may pair those
  error boundaries with one interrupting timer, message, or signal boundary
  on that same owner, and one same-package `callActivity` owner may now
  expose that same bounded mixed-owner shape
- bounded message-task execution with one `receiveTask` message wait shell,
  one `sendTask` host-dispatch shell, and one `scriptTask`
  host-dispatch shell that preserves bounded script metadata
- non-executable DMN document-snapshot preservation for top-level
  `itemDefinition` metadata plus one bounded direct `itemComponent`
  placeholder layer
- non-executable DMN document-snapshot preservation for top-level
  `inputData` metadata plus one optional direct `variable` placeholder
  layer
- non-executable DMN document-snapshot preservation for top-level
  `knowledgeSource` metadata
- non-executable DMN document-snapshot preservation for top-level
  `decisionService` metadata
- non-executable DMN document-snapshot preservation for top-level
  `businessKnowledgeModel` metadata
- non-executable DMN document-snapshot preservation for top-level
  `organizationUnit` and `performanceIndicator` metadata
- non-executable DMN document-snapshot preservation for top-level
  `textAnnotation` metadata plus one direct text payload
- non-executable DMN document-snapshot preservation for top-level
  `association` metadata plus bounded `associationDirection`, `sourceRef`,
  and `targetRef` placeholders
- non-executable DMN document-snapshot preservation for top-level
  `elementCollection` and `group` metadata
- non-executable DMN document-snapshot preservation for top-level
  `dmndi:DMNDI` metadata plus one direct `DMNDiagram` placeholder layer and
  direct `DMNShape` / `DMNEdge` placeholder metadata bounded to optional
  `id` plus `dmnElementRef`, plus one optional direct
  `DMNShape.isListedInputData` boolean, plus one optional direct
  `DMNShape.isCollapsed` boolean, plus one optional direct `dc:Bounds`
  placeholder under `DMNShape` bounded to one optional x/y/width/height
  contract, plus one repeated direct `di:waypoint` placeholder list under
  `DMNEdge` bounded to optional x/y pairs, plus one optional direct
  `DMNLabel` placeholder bounded to one optional label id plus one
  optional direct `dc:Bounds` placeholder plus one optional direct text
  payload, plus one optional direct
  `DMNDecisionServiceDividerLine` placeholder under `DMNShape` bounded to
  one repeated direct `di:waypoint` placeholder list with optional x/y
  pairs
- executable DMN clause metadata preservation for bounded `inputExpression`
  and `output` `typeRef` fields, without widening into item-definition
  resolution
- stable diagnostic surfaces that power `qianji lint --bpmn` and
  `qianji lint --dmn`

## Structural Notes

- Medium or complex features should stay folder-first.
- `src/lint/bpmn/` is the current BPMN lint owner for entry dispatch,
  document and topology guidance, reference and identity mapping, execution
  families, and unexpected-error fallback.
- `src/lint/dmn/` is the current DMN lint owner for entry dispatch,
  document guidance, contract guidance, snapshot helpers, decision helpers,
  evidence mapping, and unexpected-error fallback.
- `mod.rs` files are interface seams only and should not regrow hidden
  implementation buckets.
- Package-level OMG alignment notes live under [docs/README.md](docs/README.md).

## Non-Goals

- This crate does not promise full BPMN or DMN parity yet.
- Broader unstructured inclusive gateways and broader FEEL/script-backed
  gateway conditions remain outside the current BPMN subset.
- correlations and broader collaboration-aware message routing remain
  outside the current BPMN subset.
- Compensation event subprocesses and broader throw-compensation forms
  remain outside the current BPMN subset.
- Broader call-activity event families, transaction-shell
  message/signal/timer boundaries that exceed one interrupting
  timer/message/signal boundary, exceed one interrupting cancel boundary, or
  otherwise exceed the bounded same-owner external-plus-cancel-plus-error
  subset, broader mixed boundary families on same-package
  `callActivity` owners or embedded subprocess owners beyond one
  interrupting timer/message/signal boundary plus one or more interrupting
  error boundaries, broader non-interrupting boundary families on
  subprocess-like owners, and broader top-level error propagation beyond the
  bounded terminal error-end path remain outside the current BPMN subset.
- Adapter-specific orchestration belongs in higher layers such as
  `xiuxian-qianji`, not in the engine core.
- DMN widening should stay incremental and preserve LLM-friendly repair
  guidance rather than trading precision for broad but lossy support.
- Item-definition resolution, broader DRD execution, and executable type
  semantics remain outside the current DMN subset even though bounded
  non-executable `itemDefinition`, `inputData`, `knowledgeSource`,
  `decisionService`, `businessKnowledgeModel`, `organizationUnit`, and
  `performanceIndicator` snapshot metadata, bounded non-executable
  `textAnnotation` metadata plus one direct text payload, bounded
  non-executable `association`, `elementCollection`, and `group` metadata,
  bounded non-executable `dmndi:DMNDI` metadata plus one direct
  `DMNDiagram` placeholder layer and direct `DMNShape` / `DMNEdge`
  placeholder metadata bounded to optional `id` plus `dmnElementRef`, plus
  one optional direct `DMNShape.isListedInputData` boolean, plus one
  optional direct `DMNShape.isCollapsed` boolean, plus one optional direct
  `dc:Bounds` placeholder under `DMNShape` bounded to one optional
  x/y/width/height contract, plus one repeated direct
  `di:waypoint` placeholder list under `DMNEdge` bounded to optional x/y
  pairs, plus one optional direct `DMNLabel` placeholder bounded to one
  optional label id plus one optional direct `dc:Bounds` placeholder plus
  one optional direct text payload, plus one optional direct
  `DMNDecisionServiceDividerLine` placeholder under `DMNShape` bounded to
  one repeated direct `di:waypoint` placeholder list with optional x/y
  pairs, and bounded executable clause `typeRef` metadata are now
  preserved.
- Trailing-lower-unit fractional duration forms such as
  `duration("PT1.5H30S")`, mixed year-month/day-time duration forms,
  fractional year-month duration literals, and broader FEEL/script-backed
  temporal functions remain outside the current DMN subset.
