# DMN Alignment Index

This module tracks how `qianji-bpmn-engine` aligns with the official
[DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

The current engine does not attempt full DMN coverage. It owns one bounded
decision-table evaluator plus parser and lint surfaces that are designed to
stay explicit about the supported subset.

## Current Modules

- [Document Snapshot](document-snapshot.md)
- [Decision Tables](decision-tables.md)
- [FEEL Subset](feel-subset.md)

## Current Package Boundary

The current DMN slice supports one bounded parser-owned document snapshot,
including preserved top-level `itemDefinition` metadata plus one bounded
direct `itemComponent` placeholder layer, preserved top-level `inputData`
metadata plus one optional direct `variable` placeholder layer, preserved
top-level `knowledgeSource` metadata, preserved top-level `decisionService`
metadata, preserved top-level `businessKnowledgeModel` metadata, preserved
top-level `organizationUnit` metadata, preserved top-level
`performanceIndicator` metadata, preserved top-level `textAnnotation`
metadata plus one direct text payload, preserved top-level `association`
metadata with bounded direction/ref placeholders, preserved top-level
`elementCollection` metadata, preserved top-level `group` metadata,
preserved top-level `dmndi:DMNDI` metadata with one direct `DMNDiagram`
placeholder layer plus direct `DMNShape` / `DMNEdge` placeholder metadata
bounded to optional `id` plus `dmnElementRef` and one optional direct
`DMNShape.isListedInputData` boolean plus one optional direct
`DMNShape.isCollapsed` boolean plus one optional direct `dc:Bounds`
placeholder under `DMNShape` bounded to one optional x/y/width/height
contract plus one repeated direct `di:waypoint` placeholder list under
`DMNEdge` bounded to optional x/y pairs plus one optional direct
`DMNLabel` placeholder bounded to one optional label id plus one optional
direct `dc:Bounds` placeholder plus one optional direct text payload plus
one optional direct `DMNDecisionServiceDividerLine` placeholder under
`DMNShape` bounded to one repeated direct `di:waypoint` placeholder list
with optional x/y pairs,
multiple bounded decisions from one DMN source, one bounded
decision-table evaluation contract with preserved clause `typeRef` metadata
on executable input/output clauses, and LLM-friendly diagnostics for
unsupported syntax.
