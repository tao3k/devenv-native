# DMN Context Expressions

This module records the current bounded direct context-expression alignment
against the official [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

## Supported Direct Context Shape

The current engine supports one direct decision-owned `<context>` body when
every direct child is a `<contextEntry>` with:

- at most one direct `<variable name="..."/>`
- exactly one direct `<literalExpression><text>...</text></literalExpression>`
  body accepted by the bounded literal-expression runtime
- an optional final unnamed context entry that returns the decision value

Named entries are evaluated in source order. Each named result is added to a
temporary local context, so later entries can reference earlier names such as
`nextAge`. The host input variables remain visible while those local names are
resolved.

Each entry body may be one of these bounded forms:

- one quoted string, number, boolean, `null`, ISO date, ISO time, ISO
  datetime, or ISO duration literal already accepted by the bounded FEEL
  literal parser
- one variable path such as `applicant.age` or `nextAge`
- one whitespace-delimited numeric path operation such as
  `applicant.age + 1` or `nextAge - 1`

Runtime output is deterministic and object-shaped. If the context has a final
unnamed entry, the result is `{ "<decision_id>": <final_value> }`. If every
entry is named, the result is `{ "<decision_id>": { <named_entries> } }`.

## Deferred Direct Context Shape

These context shapes remain outside the current bounded surface:

- nested contexts or broader boxed expressions inside context entries
- invocation, function-definition, relation, or nested-list semantics inside
  entries
- non-final unnamed context entries
- script-backed context-entry evaluation
- dependency resolution across imported DMN files
- implicit conversion of context entries into guessed decision-table rules

The DMN linter emits `dmn.unsupported_context_expression_subset` when a direct
context entry exceeds the executable item subset and
`dmn.unsupported_context_child` when the direct context contains a child
outside the bounded context-entry shape.
