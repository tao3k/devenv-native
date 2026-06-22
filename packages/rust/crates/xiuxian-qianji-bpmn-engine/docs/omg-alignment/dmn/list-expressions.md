# DMN List Expressions

This module records the current bounded direct list-expression alignment
against the official [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

## Supported Direct List Shape

The current engine supports one direct decision-owned `<list>` body when every
direct child is a direct `<literalExpression><text>` item accepted by the
bounded literal-expression runtime.

Each item may be one of these bounded forms:

- one quoted string, number, boolean, `null`, ISO date, ISO time, ISO
  datetime, or ISO duration literal already accepted by the bounded FEEL
  literal parser
- one variable path such as `applicant.age`
- one whitespace-delimited numeric path operation such as
  `applicant.age + 1` or `applicant.age - 1`

Runtime output is deterministic and object-shaped:
`{ "<decision_id>": [<evaluated_items>...] }`. This keeps BPMN
`businessRuleTask` result merging compatible with the existing object-output
contract used by decision tables and direct literal expressions.

## Deferred Direct List Shape

These list shapes remain outside the current bounded surface:

- nested lists or broader boxed expressions as direct list children
- FEEL contexts, invocations, function definitions, and relations inside list
  items
- script-backed list-item evaluation
- dependency resolution across imported DMN files
- implicit conversion of list items into guessed decision-table rules

The DMN linter emits `dmn.unsupported_list_expression_subset` when a direct
list item exceeds the executable item subset and `dmn.unsupported_list_child`
when the direct list contains a non-literal child.
