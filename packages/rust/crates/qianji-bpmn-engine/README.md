# qianji-bpmn-engine

Bounded BPMN and DMN workflow engine ownership for Qianji.

## Responsibility

`qianji-bpmn-engine` owns the engine-side core for:

- BPMN parsing, bounded execution, and host-blocked workflow runtime
- DMN parsing, bounded evaluation, and LLM-friendly lint diagnostics
- bounded ISO date, datetime, time, and signed day-time or year-month
  duration decision predicates
- checkpoint codecs plus distributed Valkey-backed checkpoint ownership
- lightweight local checkpoint persistence behind the `sqlite` feature
- bounded `exclusiveGateway` condition routing with simple boolean-path or
  numeric-comparison `sequenceFlow` conditions plus one optional `default`
  branch
- bounded structured `inclusiveGateway` split/join routing with the same
  bounded condition subset plus one matching linear join fragment
- bounded transaction cancel and error routing, including one explicit
  transaction-cancel compensation subset with reverse completion replay plus
  one synchronous throw-compensation end-event subset with either explicit
  `activityRef` targeting or bounded default replay plus one synchronous
  throw-compensation intermediate-event subset with either explicit
  `activityRef` targeting or bounded default replay
- bounded message-task execution with one `receiveTask` message wait shell
  and one `sendTask` host-dispatch shell
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

## Non-Goals

- This crate does not promise full BPMN or DMN parity yet.
- Broader unstructured inclusive gateways and broader FEEL/script-backed
  gateway conditions remain outside the current BPMN subset.
- `scriptTask`, correlations, and broader collaboration-aware message
  routing remain outside the current BPMN subset.
- Compensation event subprocesses, asynchronous throw-compensation end
  events, asynchronous throw-compensation intermediate events, and broader
  throw-compensation forms remain outside the current BPMN subset.
- Adapter-specific orchestration belongs in higher layers such as
  `xiuxian-qianji`, not in the engine core.
- DMN widening should stay incremental and preserve LLM-friendly repair
  guidance rather than trading precision for broad but lossy support.
- Trailing-lower-unit fractional duration forms such as
  `duration("PT1.5H30S")`, mixed year-month/day-time duration forms,
  fractional year-month duration literals, and broader FEEL/script-backed
  temporal functions remain outside the current DMN subset.
