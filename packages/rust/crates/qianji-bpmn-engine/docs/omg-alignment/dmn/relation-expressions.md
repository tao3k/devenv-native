# DMN Relation Expressions

This module records the current bounded direct relation-expression alignment
against the official [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

## Supported Direct Relation Shape

The current engine supports one direct decision-owned `<relation>` body when
the relation has:

- one or more direct `<column>` elements
- zero or more direct `<row>` elements
- exactly one direct `<literalExpression><text>...</text></literalExpression>`
  cell per column in each row
- cell text accepted by the bounded literal-expression runtime

Column output keys are deterministic. The engine uses the column `name` when it
is present and falls back to the required column `id` otherwise.

Each cell may be one of these bounded forms:

- one quoted string, number, boolean, `null`, ISO date, ISO time, ISO
  datetime, or ISO duration literal already accepted by the bounded FEEL
  literal parser
- one variable path such as `applicant.age`
- one whitespace-delimited numeric path operation such as
  `applicant.age + 1` or `applicant.age - 1`

Runtime output is deterministic and object-shaped:
`{ "<decision_id>": [{ "<column_key>": <cell_value>, ... }, ...] }`. This keeps
BPMN `businessRuleTask` result merging compatible with the existing
object-output contract used by decision tables and the other direct expression
subsets.

## Deferred Direct Relation Shape

These relation shapes remain outside the current bounded surface:

- nested relations or broader boxed expressions inside relation cells
- relation children beyond direct columns and rows
- row cells whose arity does not match the relation column count
- invocation, function-definition, context, or list semantics inside cells
- script-backed cell evaluation
- dependency resolution across imported DMN files
- implicit conversion of relation rows into guessed decision-table rules

The DMN linter emits `dmn.unsupported_relation_expression_subset` when a direct
relation cell exceeds the executable cell subset and
`dmn.unsupported_relation_child` when the direct relation contains a child
outside the bounded column/row shape.
