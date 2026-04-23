# DMN Alignment Index

This module tracks how `qianji-bpmn-engine` aligns with the official
[DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

The current engine does not attempt full DMN coverage. It owns one bounded
decision-table evaluator plus parser and lint surfaces that are designed to
stay explicit about the supported subset.

## Current Modules

- [Document Snapshot](document-snapshot.md)
- [Decision Tables](decision-tables.md)
- [Literal Expressions](literal-expressions.md)
- [List Expressions](list-expressions.md)
- [Context Expressions](context-expressions.md)
- [Relation Expressions](relation-expressions.md)
- [Function Definitions](function-definitions.md)
- [Business Knowledge Models](business-knowledge-models.md)
- [Decision Services](decision-services.md)
- [Requirement References](requirement-references.md)
- [Information Requirements](information-requirements.md)
- [FEEL Subset](feel-subset.md)

## Current Package Boundary

The current DMN slice supports one bounded parser-owned document snapshot,
including preserved top-level `itemDefinition` metadata plus one bounded
direct `itemComponent` placeholder layer, preserved top-level `inputData`
metadata plus one optional direct `variable` placeholder layer, preserved
top-level `knowledgeSource` metadata, preserved top-level `decisionService`
metadata plus direct `outputDecision`, `encapsulatedDecision`,
`inputDecision`, and `inputData` href placeholders, preserved top-level
`businessKnowledgeModel` metadata plus one direct body `literalExpression`
placeholder plus one optional invocable `variable` placeholder and one bounded
`encapsulatedLogic` function-definition placeholder, preserved top-level `organizationUnit`
metadata, preserved top-level
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
decision-owned requirement-reference snapshot surface carrying
`informationRequirement`, `knowledgeRequirement`, and `authorityRequirement`
target href placeholders, one bounded
executable direct `informationRequirement` contract carrying preserved
`requiredInput` / `requiredDecision` href placeholders on parsed decision
definitions, plus one bounded
same-source local `requiredDecision` runtime-resolution path that overlays
upstream decision outputs before evaluating the current decision, plus one
bounded same-source local `requiredInput` alias-bind path that reuses
top-level `inputData.name` plus nested `variable.name` metadata while still
requiring the caller to supply the original input object, one bounded
decision-table evaluation contract with preserved clause `typeRef` metadata
on executable input/output clauses, one bounded direct
`literalExpression` evaluator for literals, variable paths, and one
whitespace-delimited numeric `path +/- number` operation, one bounded direct
`list` evaluator whose direct children are bounded `literalExpression` items,
one bounded direct `context` evaluator whose ordered entries are bounded
`literalExpression` bodies with optional variable names and an optional final
unnamed result entry, one bounded direct `relation` evaluator whose direct
rows contain one bounded `literalExpression` cell per direct column,
non-executable direct `invocation` snapshot evidence for invoked
literal-expression text plus binding parameter/argument placeholders,
non-executable direct `functionDefinition` snapshot evidence for function kind,
formal parameters, and body literal-expression placeholders,
non-executable top-level `businessKnowledgeModel` body snapshot evidence, and
non-executable top-level `decisionService` reference snapshot evidence, plus
LLM-friendly diagnostics for unsupported syntax.
