# DMN Invocations

This module records the bounded `xiuxian-qianji-bpmn-engine` contract for direct DMN
`invocation` expressions.

## Current Support

The current engine supports one bounded local invocation seam:

- one direct decision-owned `<invocation>` may execute locally when the invoked
  expression resolves to exactly one same-source top-level
  `businessKnowledgeModel`
- resolution may use either the BKM id or the BKM invocable `variable.name`
- each direct binding may expose one named parameter and one supported direct
  literal-expression argument
- the target BKM must expose one bounded `encapsulatedLogic`
  `functionDefinition` whose body is itself one supported direct
  `literalExpression`
- when the decision also carries executable same-source `requiredKnowledge`
  edges, the invocation target must match those preserved top-level BKM ids

Missing, ambiguous, or mismatched local BKM targets fail explicitly instead of
falling back to host-side FEEL evaluation.

## Runtime Boundary

The evaluator still does not implement:

- imported callable resolution
- nested or indirect invocation chains
- broader FEEL function bodies or non-literal binding arguments
- externally defined Java, PMML, or other non-local callable targets
- standalone public DMN evaluation that bypasses the package-owned BKM
  registry

## Repair Guidance

Keep executable invocation models bounded to one same-source BKM with explicit
local bindings and one direct literal-expression body when current runtime
support is required. Otherwise preserve the invocation in the snapshot and lint
surface and let later FEEL or adapter work widen it deliberately.
