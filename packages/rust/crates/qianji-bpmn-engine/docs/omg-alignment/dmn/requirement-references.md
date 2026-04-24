# DMN Requirement References

This module records the bounded `qianji-bpmn-engine` contract for
decision-owned DMN requirement references.

## Current Support

- decision-owned `informationRequirement`, `knowledgeRequirement`, and
  `authorityRequirement` counts are preserved in the non-executable decision
  snapshot
- direct target references are preserved as `requirement_kind`,
  `reference_kind`, and `href` placeholders
- preserved references include direct `requiredInput`, `requiredDecision`,
  `requiredKnowledge`, and `requiredAuthority` targets, including
  `authorityRequirement` branches that point at decision or input dependencies
- `qianji lint --dmn` reports unsupported requirement-only decisions with
  snapshot evidence that is stable enough for LLM repair flows to preserve
  dependency edges instead of inventing rules

The bounded reference shape follows the
[OMG DMN machine-readable schema](https://www.omg.org/spec/DMN/machine-readable)
where requirement targets are DMN element references with `href` payloads.

## Runtime Boundary

The evaluator now consumes one bounded subset of executable
`informationRequirement` references: direct same-source `requiredDecision`
recursion plus one bounded same-source `requiredInput` alias bind when
parse-time `inputData` metadata is available. The evaluator also now consumes
one bounded executable `knowledgeRequirement` subset: when a decision already
has direct local invocation logic, same-source `requiredKnowledge` hrefs can
constrain that invocation target to the explicitly declared top-level BKM ids.
Broader href resolution, import resolution, decision-service orchestration,
standalone BKM execution, authority execution, and dependency scheduling
remain deferred. `knowledgeRequirement` still does not auto-materialize a
missing local decision body by itself.

## Repair Guidance

When a model uses requirement edges without a local executable decision table,
preserve the decision id, name, parent requirement kind, target reference kind,
and href. Only add a bounded `decisionTable` when the missing local decision
logic is explicit and lossless. Otherwise keep the source as a non-executable
DRD dependency artifact and report unsupported requirement-only execution.
