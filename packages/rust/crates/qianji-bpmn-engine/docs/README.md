# qianji-bpmn-engine Docs

This directory holds package-level documentation for the bounded
`qianji-bpmn-engine` surface.

The current documentation baseline is organized around official OMG standards:

- BPMN alignment tracks the formal
  [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).
- DMN alignment tracks the current formal
  [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

Use these modules as the package-local source of truth for what the crate
currently supports, what is still intentionally deferred, and where new slices
should extend the bounded surface.

## Modules

- [OMG Alignment Index](omg-alignment/README.md)
- [BPMN Alignment Index](omg-alignment/bpmn/README.md)
- [BPMN Events and Boundaries](omg-alignment/bpmn/events-and-boundaries.md)
- [BPMN Loops and Multi-Instance](omg-alignment/bpmn/loops-and-multi-instance.md)
- [DMN Alignment Index](omg-alignment/dmn/README.md)
- [DMN Decision Tables](omg-alignment/dmn/decision-tables.md)
- [DMN FEEL Subset](omg-alignment/dmn/feel-subset.md)
