# DMN Decision Services

This module records the bounded `qianji-bpmn-engine` contract for top-level
DMN `decisionService` elements.

## Current Support

- top-level `decisionService` metadata is preserved in the non-executable
  document snapshot
- preserved metadata includes the optional decision-service id, optional name,
  and direct `outputDecision`, `encapsulatedDecision`, `inputDecision`, and
  `inputData` href placeholders
- `qianji lint --dmn` reports unsupported decision-service documents with
  snapshot evidence that is stable enough for LLM repair flows to preserve the
  service contract instead of inventing rules

The bounded reference shape follows the
[OMG DMN machine-readable schema](https://www.omg.org/spec/DMN/machine-readable)
where the direct decision-service children are DMN element references with
`href` payloads.

## Runtime Boundary

The evaluator does not execute top-level `decisionService` elements yet. This
slice deliberately avoids output-decision resolution, import resolution, DRD
dependency execution, and decision-service orchestration.

## Repair Guidance

When a model uses a top-level `decisionService`, preserve the service id, name,
and direct reference hrefs. Only expose or translate it into bounded
`decisionTable` decisions when the referenced decisions and rule mappings are
explicit and lossless. Otherwise keep the source as a non-executable artifact
and report unsupported decision-service execution.
