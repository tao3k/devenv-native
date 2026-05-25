# DMN FEEL Subset

This module records the current bounded FEEL alignment against the official
[DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

## Supported Literal and Predicate Families

The current engine supports:

- string, number, boolean, and null equality checks
- wildcard matching
- bounded numeric unary comparisons and ranges
- ISO date comparisons and ranges
- ISO local and RFC3339 offset-aware datetime comparisons and ranges
- ISO time comparisons and ranges
- signed ISO 8601 day-time and year-month duration comparisons and ranges
- bounded day-time duration fractions such as `duration("P1.5D")` and
  `duration("PT1.5H")`
- direct decision-owned literal expressions when the expression text is one
  supported literal, one variable path, or one whitespace-delimited numeric
  `path +/- number` operation
- direct decision-owned list expressions when every direct list child is a
  supported bounded literal-expression item
- direct decision-owned context expressions when every direct context entry has
  one supported bounded literal-expression body, optional variable metadata,
  and any unnamed result entry is final
- direct decision-owned relation expressions when every direct row has one
  supported bounded literal-expression cell per direct column

## Deferred FEEL Surface

The current engine still defers:

- broader direct-expression arithmetic beyond one `path +/- number` operation
- nested or mixed boxed-expression list items beyond direct literal-expression
  children
- nested or mixed boxed-expression context entries beyond direct
  literal-expression bodies
- nested or mixed boxed-expression relation cells beyond direct
  literal-expression bodies
- broader built-in FEEL functions
- mixed-family duration handling
- trailing-lower-unit fractional duration literals such as
  `duration("PT1.5H30S")`
- fractional year-month duration literals such as `duration("P1.5Y")`
- full context, list, relation, and function semantics beyond the bounded
  direct expression subsets
