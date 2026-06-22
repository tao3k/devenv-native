# DMN Decision Services

This module records the bounded `xiuxian-qianji-bpmn-engine` contract for top-level
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
`name`, `namespace`, `locationURI`, and `importType` metadata. Bundle loading
can preserve source-root and import metadata for those imported DMN sources,
but it keeps them out of executable decision, input-data,
business-knowledge-model, and decision-service registries. The package model
now has a non-executable, source-scoped import registry that keeps the declaring
`source_id`, import alias, imported namespace, location URI, and import type
separate; future cross-document lookup must resolve through that contract
instead of treating aliases, namespaces, and source ids as interchangeable.
The registry can be queried deterministically by declaring source plus alias,
namespace, or location URI, and it reports ambiguous selectors instead of
choosing one imported dependency implicitly.
Bundled DMN source roots are also preserved in a separate non-executable
registry so an imported namespace can be matched to a package-owned source
root without treating that source id, namespace, or import alias as the same
identifier. That match is metadata-only: it does not follow `locationURI`,
parse imported decisions, or make imported decision-service orchestration
executable.
Package consumers can also request one owned import-to-source binding report
for every registered import. The report preserves unbound imports as metadata
observations and rejects ambiguous namespace targets instead of selecting a
source root implicitly.

## Repair Guidance

When a model uses a top-level `decisionService`, preserve the service id, name,
and direct reference hrefs. Only route it into local execution when it is one
same-source service whose direct local `outputDecision` targets are already
executable under the bounded engine contract.
Otherwise keep the source as a non-executable artifact and report unsupported
decision-service execution instead of fabricating decision-table logic or DRD
orchestration.
