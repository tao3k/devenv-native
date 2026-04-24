# DMN Decision Tables

This module records the current bounded decision-table alignment against the
official [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

## Supported Table Shape

The current engine supports one bounded decision-table family:

- parser-owned document snapshots with multiple bounded decisions from one DMN
  source
- one bounded decision-table per decision
- preserved executable clause metadata for `inputExpression typeRef` and
  `output typeRef`
- hit policies `UNIQUE` and `COLLECT`
- bounded wildcard matching, literal equality, numeric unary comparisons, and
  bounded numeric/date/time/datetime ranges

## Deferred Table Shape

These decision-table shapes remain outside the current bounded surface:

- broader hit policies beyond `UNIQUE` and `COLLECT`
- richer boxed expressions and invocation chains beyond the separately
  supported direct literal-expression, list-expression, context-expression,
  and relation-expression subsets
- broader FEEL list, context, function, and temporal operators
- item-definition resolution and executable type semantics beyond preserved
  clause metadata
- full DMN import and dependency handling
