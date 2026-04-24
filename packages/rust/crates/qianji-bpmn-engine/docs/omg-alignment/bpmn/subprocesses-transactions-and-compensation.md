# BPMN Subprocesses, Transactions, and Compensation

This module records the current bounded nested-scope alignment against the
official [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Supported Nested Scope Families

The current engine supports these bounded nested-scope families:

- embedded `subProcess` with exactly one nested `startEvent` and at least one
  nested `endEvent`
- same-package `callActivity` that targets another executable process in the
  same parsed package
- `transaction` shell with exactly one nested `startEvent` and at least one
  nested `endEvent`
- interrupting timer, message, or signal boundaries on embedded subprocess,
  same-package call activity, and transaction owners
- interrupting error boundaries on embedded subprocess, same-package call
  activity, and transaction owners
- one interrupting cancel boundary on one bounded transaction owner
- bounded mixed transaction owners that combine one external interrupting
  boundary with one cancel boundary, one or more error boundaries, or both
- bounded mixed embedded subprocess and call-activity owners that combine one
  external interrupting boundary with one or more error boundaries

## Compensation Support

The current compensation slice is bounded to transaction-owned compensation:

- compensation boundary events attach to already supported host-blocking
  activities inside one bounded transaction shell
- compensation handlers are detached host-blocking activities marked with
  `isForCompensation="true"` and reached through one association
- transaction cancel can run compensation before routing the parent cancel
  boundary
- throw-compensation end events can target one explicit `activityRef` or omit
  `activityRef` for bounded reverse-completion replay
- throw-compensation intermediate events can run targeted or default replay
  before normal routing resumes
- throw-compensation events may stay synchronous or set
  `waitForCompletion="false"` for the landed asynchronous bounded paths

## Runtime Guarantees

Within that bounded slice, the runtime guarantees:

- parent process state is suspended while embedded subprocess, call activity,
  or transaction child state runs
- interrupting parent boundaries can restore the parent scope and cancel the
  active child scope deterministically
- error ends route only through matching parent error boundaries attached to
  the same owner, including catch-all boundaries with omitted `errorRef`
- transaction cancel rolls back the transaction variables before routing the
  cancel boundary
- detached asynchronous compensation queues can continue after the parent path
  resumes
- compensation replay order is deterministic and uses reverse completion order
  for default replay

## Deferred Nested Scope Semantics

These nested-scope shapes remain outside the bounded surface:

- event subprocesses, including compensation event subprocesses
- recursive call-activity chains
- non-interrupting timer, message, or signal boundaries on embedded
  subprocess, call activity, or transaction owners
- more than one cancel boundary on a transaction owner
- broader compensation handlers outside the bounded transaction shell
- broader compensation throwing outside the bounded transaction-owned end and
  intermediate event paths
- ad hoc child-process resolution outside the same parsed BPMN package

## Alignment Notes

The nested-scope model is intentionally package-local and checkpoint-friendly.
It materializes child execution as explicit runtime frames rather than as an
opaque object graph, so the state can be serialized, resumed, linted, and
adapted by host-owned Qianji orchestration without leaking BPMN internals into
the host crate.
