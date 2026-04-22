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

## Deferred Loop Semantics

These loop shapes remain deferred:

- broader `standardLoopCharacteristics` beyond the current boolean-path and
  loop-maximum subset
- broader multi-instance event throwing and completion behaviors from the OMG
  surface
- collection mediation beyond the current bounded JSON variable overlay model
