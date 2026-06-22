# DMN Literal Expressions

This module records the current bounded direct literal-expression alignment
against the official [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

## Supported Direct Expression Shape

The current engine supports one direct decision-owned
`<literalExpression><text>` body when the expression text is one of these
bounded forms:

- one quoted string, number, boolean, `null`, ISO date, ISO time, ISO
  datetime, or ISO duration literal already accepted by the bounded FEEL
  literal parser
- one variable path such as `applicant.age`
- one whitespace-delimited numeric path operation such as
  `applicant.age + 1` or `applicant.age - 1`

Runtime output is deterministic and object-shaped:
`{ "<decision_id>": <evaluated_value> }`. This keeps BPMN
`businessRuleTask` result merging compatible with the existing object-output
contract used by decision tables.

## Deferred Direct Expression Shape

These direct expression shapes remain outside the current bounded surface:

- broader FEEL arithmetic beyond one `path +/- number` operation
- FEEL contexts, nested or mixed lists, invocations, function definitions, and
  relation semantics beyond the separately supported direct relation subset
- script-backed expression evaluation
- dependency resolution across imported DMN files
- implicit conversion of complex direct expressions into guessed decision-table
  rules

The DMN linter emits `dmn.unsupported_literal_expression_subset` for direct
literal-expression text that exceeds this executable subset, with repair
guidance intended for LLM-assisted source fixes.
