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

## Deferred FEEL Surface

The current engine still defers:

- broader built-in FEEL functions
- mixed-family duration handling
- trailing-lower-unit fractional duration literals such as
  `duration("PT1.5H30S")`
- fractional year-month duration literals such as `duration("P1.5Y")`
- full context, list, and function semantics
