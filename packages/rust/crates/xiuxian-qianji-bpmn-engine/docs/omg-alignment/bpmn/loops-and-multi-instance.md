# BPMN Loops and Multi-Instance

This module records the current bounded loop alignment against the official
[BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Supported Loop Families

The current engine supports:

- bounded `standardLoopCharacteristics` on one service, user, manual, or
  business-rule task family
- bounded sequential `multiInstanceLoopCharacteristics isSequential="true"`
  with integer `loopCardinality`
- bounded parallel `multiInstanceLoopCharacteristics` with omitted or
  `isSequential="false"` plus integer `loopCardinality`
- one bounded `completionCondition` subset over simple boolean paths or
  bounded counter comparisons
- one bounded collection-backed input and output binding subset using
  `loopDataInputRef`, `inputDataItem`, optional `loopDataOutputRef`, and
  `outputDataItem`

## Runtime Guarantees

Within that bounded slice, the runtime now guarantees:

- deterministic re-entry for standard loop and sequential multi-instance owners
- zero-cardinality skip for sequential and parallel multi-instance owners
- bounded early-stop semantics through `completionCondition`
- deterministic output aggregation for the bounded collection-backed subset
- interrupting boundary cleanup on bounded repeating task owners
- owner-level non-interrupting boundary support on bounded standard-loop,
  sequential multi-instance, and parallel multi-instance task owners

## Lint-Time Cycle Risk Checks

Sequence-flow cycles are allowed only when the workflow makes progress
explicit. `qianji lint` reports `bpmn.loop_risk.unbounded_control_cycle` when a
cycle can re-enter host or user work without a complete progress contract.

The current check is intentionally conservative for LLM-authored interactive
workflows:

- a cycle must have an exit path, usually an unconditional default branch
- gateway route variables used by the cycle must be declared by
  `dataOutputAssociation targetRef` on a task inside the same cycle
- if an in-cycle service task emits prompt-like outputs such as
  `currentQuestion` or `currentChoices`, user-task outputs from the same cycle
  must feed back through that service task's `dataInputAssociation sourceRef`

The diagnostic is source-span-aware and keeps natural-language guidance in the
diagnostic layer only. The LLM-facing output includes the issue code, title,
file span, source line, caret label, one-line `Help`, and one-line `Contract`.
The repair layer is a git-diff-style proposed patch, not an action list or a
large template.

For example, a missing feedback input should report the unsafe line and propose
only the minimal hunk:

```diff
@@ -12,1 +12,1 @@
-          <dataInputAssociation><sourceRef></sourceRef></dataInputAssociation>
+          <dataInputAssociation><sourceRef>answer</sourceRef></dataInputAssociation>
```

Structured lint consumers can still read the underlying `line_fixes` metadata
from the JSON report; text repair flows should use the compact diagnostic plus
proposed patch.

## Deferred Loop Semantics

These loop shapes remain deferred:

- broader `standardLoopCharacteristics` beyond the current boolean-path and
  loop-maximum subset
- broader multi-instance event throwing and completion behaviors from the OMG
  surface
- collection mediation beyond the current bounded JSON variable overlay model
