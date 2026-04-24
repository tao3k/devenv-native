# DMN Decision Services

This module records the bounded `qianji-bpmn-engine` contract for top-level
DMN `decisionService` elements.

## Current Support

- top-level `decisionService` metadata is preserved in the non-executable
  document snapshot
- preserved metadata includes the optional decision-service id, optional name,
  and direct `outputDecision`, `encapsulatedDecision`, `inputDecision`, and
  `inputData` href placeholders
- parser-owned bundle loading now materializes one bounded package-owned
  same-source decision-service registry from that preserved metadata
- local BPMN business-rule runtime can now resolve one same-source
  `decisionService` reference as a thin local alias to one or more direct
  local `outputDecision` targets
- before that bounded local alias runs, the evaluator now also validates every
  preserved same-source `encapsulatedDecision`, `inputDecision`, and
  `inputData` exposure href against the local package registries so broken
  service declarations fail explicitly instead of being silently ignored
- `qianji lint --dmn` reports unsupported decision-service documents with
  snapshot evidence that is stable enough for LLM repair flows to preserve the
  service contract instead of inventing rules

The bounded reference shape follows the
[OMG DMN machine-readable schema](https://www.omg.org/spec/DMN/machine-readable)
where the direct decision-service children are DMN element references with
`href` payloads.

## Runtime Boundary

The evaluator still does not implement general decision-service orchestration.
This slice only supports one same-source decision service whose preserved
direct `outputDecision` list contains one or more local `#decisionId` targets.
One output target preserves the existing single-output result shape; multiple
output targets are evaluated in source order and merged into one object-shaped
context. Other preserved exposure refs are consumed only as same-source closure
validation, not as executable orchestration. Imported hrefs, broader DRD
planning, and general decision-service orchestration all remain deferred.
Top-level DMN imports are now preserved in document snapshots with bounded
`name`, `namespace`, `locationURI`, and `importType` metadata, but executable
package loading still rejects them until an explicit import-resolution registry
can map those metadata fields to package-owned DMN sources.

## Repair Guidance

When a model uses a top-level `decisionService`, preserve the service id, name,
and direct reference hrefs. Only route it into local execution when it is one
same-source service whose direct local `outputDecision` targets are already
executable under the bounded engine contract.
Otherwise keep the source as a non-executable artifact and report unsupported
decision-service execution instead of fabricating decision-table logic or DRD
orchestration.
