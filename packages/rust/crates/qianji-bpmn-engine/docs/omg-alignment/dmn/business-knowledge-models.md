# DMN Business Knowledge Models

This module records the bounded `qianji-bpmn-engine` contract for top-level
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
- `qianji lint --dmn` reports unsupported BKM-only documents with snapshot
  evidence that is stable enough for LLM repair flows to preserve the bounded
  BKM invocable surface instead of inventing rules

## Runtime Boundary

The evaluator does not execute top-level `businessKnowledgeModel` elements yet.
This slice deliberately avoids BKM body evaluation, invocation binding, import
resolution, and DRD dependency execution. The immediate blocker is no longer
parser ownership of the invocable contract; it is runtime ownership. The
bounded parser now preserves one invocable `variable` / `encapsulatedLogic`
placeholder contract and one package-owned same-source BKM registry, but
runtime still does not execute that callable knowledge surface or consume
`requiredKnowledge` automatically.

## Repair Guidance

When a model uses a top-level `businessKnowledgeModel`, preserve the BKM id,
name, invocable `variable`, preserved `encapsulatedLogic`, and direct body
expression when present. Do not flatten a BKM into guessed local decision-table
rules just because a `requiredKnowledge` edge points at it. Only translate it
into a bounded executable decision when the missing decision contract, callable
binding semantics, and rule mapping are explicit and lossless. Otherwise keep
the source as a non-executable artifact and report unsupported BKM execution.
