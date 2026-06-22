# DMN Function Definitions

This module records the bounded `xiuxian-qianji-bpmn-engine` contract for direct
decision-owned DMN `functionDefinition` elements.

## Current Support

- direct decision-owned `functionDefinition` metadata is preserved in the
  non-executable document snapshot
- preserved metadata includes the optional function-definition id, optional
  `kind`, direct `formalParameter` id/name/typeRef placeholders, and one direct
  body `literalExpression` id/typeRef/text placeholder
- `qianji lint --dmn` reports unsupported direct function-definition decisions
  with snapshot evidence that is stable enough for LLM repair flows to preserve
  parameters and body text

## Runtime Boundary

The evaluator does not execute direct `functionDefinition` decisions yet. This
slice deliberately avoids function calls, parameter binding, business knowledge
model execution, import resolution, and DRD dependency execution.

## Repair Guidance

When a model uses a direct `functionDefinition`, preserve the decision id,
function kind, formal parameters, and body expression. Only replace it with a
bounded `decisionTable` when the parameter-to-clause mapping and body-to-rule
mapping are explicit and lossless. Otherwise keep the source as a
non-executable placeholder and report unsupported function-definition execution.
