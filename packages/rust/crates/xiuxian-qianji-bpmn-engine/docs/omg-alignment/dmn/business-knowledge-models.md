# DMN Business Knowledge Models

This module records the bounded `xiuxian-qianji-bpmn-engine` contract for top-level
DMN `businessKnowledgeModel` elements.

## Current Support

- top-level `businessKnowledgeModel` metadata is preserved in the
  non-executable document snapshot
- preserved metadata includes the optional BKM id, optional name, one optional
  invocable `variable` placeholder, one bounded `encapsulatedLogic`
  placeholder reusing the current function-definition snapshot contract, and
  the earlier direct body `literalExpression` placeholder seam
- parser-owned bundle loading can now materialize one bounded package-owned
  BKM registry from same-source top-level `businessKnowledgeModel` metadata so
  later runtime slices can resolve local BKM ids without re-reading snapshots
- the local evaluator can now execute one bounded direct invocation seam when a
  decision-owned `<invocation>` resolves to exactly one same-source top-level
  BKM by id or invocable `variable` name, each binding exposes one simple named
  parameter plus one supported literal-expression argument, the target
  `encapsulatedLogic` provides one supported direct literal-expression body,
  and any direct same-source executable `<requiredKnowledge>` edges preserved
  on the decision also resolve to that same target
- `qianji lint --dmn` reports unsupported BKM-only documents with snapshot
  evidence that is stable enough for LLM repair flows to preserve the bounded
  BKM invocable surface instead of inventing rules

## Runtime Boundary

The evaluator still does not execute top-level `businessKnowledgeModel`
elements as standalone DRD nodes, and it still does not consume
`requiredKnowledge` automatically. The landed runtime seam is narrower:
one decision-owned direct `<invocation>` may call one same-source BKM through
the package-owned registry, bind explicit named parameters through supported
literal-expression arguments, evaluate one supported direct
`encapsulatedLogic` literal-expression body, and, when executable
`<requiredKnowledge>` edges are present on the decision, restrict the callable
target to those declared same-source BKM ids. Broader BKM body evaluation,
imports, knowledge-requirement recursion, decision-service calls, and broader
FEEL callable semantics remain deferred.

## Repair Guidance

When a model uses a top-level `businessKnowledgeModel`, preserve the BKM id,
name, invocable `variable`, preserved `encapsulatedLogic`, and direct body
expression when present. Do not flatten a BKM into guessed local decision-table
rules just because a `requiredKnowledge` edge points at it. If one invocation
should execute locally, make the same-source BKM target explicit, keep each
binding parameter named, reduce the callable body plus binding arguments to the
supported bounded literal-expression subset, and keep any same-source
`requiredKnowledge` hrefs aligned with that callable target. Broader BKM or
`requiredKnowledge` execution should remain non-executable until that contract
is explicit and lossless.
