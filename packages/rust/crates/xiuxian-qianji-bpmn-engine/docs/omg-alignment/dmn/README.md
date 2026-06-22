# DMN Alignment Index

This module tracks how `xiuxian-qianji-bpmn-engine` aligns with the official
[DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

The current engine does not attempt full DMN coverage. It owns one bounded
decision-table evaluator plus parser and lint surfaces that are designed to
stay explicit about the supported subset.

The source-backed clause registry for these notes lives in
[DMN Official Source Map](spec-source-map.md).

## Current Modules

- [Official Source Map](spec-source-map.md)
- [Document Snapshot](document-snapshot.md)
- [Decision Tables](decision-tables.md)
- [Literal Expressions](literal-expressions.md)
- [Invocations](invocations.md)
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
including package-owned non-executable DMN source-root metadata with source
id, root id, root name, DMN business namespace, model namespace URI, and
model-version hint, preserved top-level `import` metadata with bounded `name`,
`namespace`, `locationURI`, and `importType` placeholders plus one
non-executable package-owned import registry that preserves the declaring
source id separately from import alias, imported namespace, location URI, and
import type and exposes deterministic source-scoped lookup by alias,
namespace, or location URI plus metadata-only bundle loading for imported DMN
sources that preserves source/import registries without populating executable
decision registries, preserved
top-level `itemDefinition` metadata plus one bounded
direct `itemComponent` placeholder layer, preserved top-level `inputData`
metadata plus one optional direct `variable` placeholder layer, preserved
top-level `knowledgeSource` metadata, preserved top-level `decisionService`
metadata plus direct `outputDecision`, `encapsulatedDecision`,
`inputDecision`, and `inputData` href placeholders plus one bounded
package-owned same-source decision-service registry, preserved top-level
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
rows contain one bounded `literalExpression` cell per direct column, one
bounded direct `invocation` evaluator whose invoked text resolves to exactly
one same-source top-level `businessKnowledgeModel` by id or invocable
`variable` name, whose direct bindings expose simple named parameters plus
supported literal-expression arguments, whose target `encapsulatedLogic`
provides one supported direct literal-expression body, and whose target must
also match any preserved executable same-source `requiredKnowledge` edges on
that decision, plus one bounded same-source local `decisionService` alias path
whose preserved direct `outputDecision` list must contain one or more local
target decisions, where multiple outputs are evaluated in source order and
merged into one object-shaped context, and whose preserved same-source
`encapsulatedDecision` / `inputDecision` / `inputData` exposure refs are
consumed only as local target validation before those output decisions run,
while top-level imports remain descriptive package/snapshot metadata rather
than executable cross-document lookup inputs,
non-executable direct `functionDefinition` snapshot evidence for function kind,
formal parameters, and body literal-expression placeholders,
non-executable top-level `businessKnowledgeModel` body snapshot evidence, and
non-executable top-level `decisionService` reference snapshot evidence for the
broader unsupported service surface, plus
LLM-friendly diagnostics for unsupported syntax.
