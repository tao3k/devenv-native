# DMN Document Snapshot

This module tracks the bounded non-executable DMN document-snapshot surface
owned by `qianji-bpmn-engine`.

## Supported Snapshot Metadata

- root element metadata including DMN namespace and version hint
- counts for top-level metadata-only artifact families such as `import`,
  `itemDefinition`, `inputData`, `knowledgeSource`,
  `businessKnowledgeModel`, `decisionService`, `organizationUnit`,
  `performanceIndicator`, `textAnnotation`, `association`,
  `elementCollection`, `group`, and `dmndi:DMNDI`
- decision-header counts for bounded requirement, governance, and unsupported
  direct-expression constructs
- decision-owned direct requirement reference metadata including parent
  requirement kind, direct reference kind, and `href`
- decision-owned direct `invocation` metadata including the direct invoked
  literal-expression text plus direct binding parameter and argument
  literal-expression placeholders
- decision-owned direct `functionDefinition` metadata including function kind,
  direct formal-parameter placeholders, and one direct body literal-expression
  placeholder
- top-level `itemDefinition` metadata including bounded `id`, `name`,
  `typeRef`, `isCollection`, and one direct `itemComponent` placeholder
  layer
- top-level `inputData` metadata including bounded `id`, `name`, and one
  optional direct `variable` placeholder with bounded `id`, `name`, and
  `typeRef`
- top-level `knowledgeSource` metadata including bounded `id` and `name`
- top-level `businessKnowledgeModel` metadata including bounded `id`, `name`,
  and one direct body `literalExpression` placeholder with bounded
  `id`/`typeRef`/text metadata
- top-level `decisionService` metadata including bounded `id`, `name`, and
  direct `outputDecision`, `encapsulatedDecision`, `inputDecision`, and
  `inputData` href placeholders
- top-level `organizationUnit` metadata including bounded `id` and `name`
- top-level `performanceIndicator` metadata including bounded `id` and
  `name`
- top-level `textAnnotation` metadata including bounded `id` and one direct
  nested text payload
- top-level `association` metadata including bounded `id`,
  `associationDirection`, and one direct `sourceRef` / `targetRef`
  placeholder layer
- top-level `elementCollection` metadata including bounded `id` and `name`
- top-level `group` metadata including bounded `id` and `name`
- top-level `dmndi:DMNDI` metadata including bounded `id` and one direct
  `DMNDiagram` placeholder layer with bounded `id`, direct `DMNShape`
  count, direct `DMNEdge` count, and direct `DMNShape` / `DMNEdge`
  placeholder metadata bounded to optional `id` plus `dmnElementRef`, plus
  one optional direct `DMNShape.isListedInputData` boolean, plus one
  optional direct `DMNShape.isCollapsed` boolean, plus one optional direct
  `dc:Bounds` placeholder under `DMNShape` bounded to one optional
  x/y/width/height contract, plus one repeated direct `di:waypoint`
  placeholder list under `DMNEdge` bounded to optional x/y pairs, plus
  one optional direct `DMNLabel` placeholder bounded to one optional label
  id plus one optional direct `dc:Bounds` placeholder plus one optional
  direct text payload, plus one optional direct
  `DMNDecisionServiceDividerLine` placeholder under `DMNShape` bounded to
  one repeated direct `di:waypoint` placeholder list with optional x/y
  pairs

## Current Boundary

- the document snapshot is descriptive only and does not make metadata-only
  DMN sources executable
- decision-owned requirement references are preserved as href placeholders
  only; this includes direct `requiredInput`, `requiredDecision`,
  `requiredKnowledge`, and `requiredAuthority`
- top-level `itemDefinition` metadata is preserved so lint, adapter, and
  later DMN type-model work can reuse stable engine-owned placeholders
- one bounded direct `itemComponent` layer is preserved under each top-level
  `itemDefinition`
- top-level `inputData` metadata is preserved so lint, adapter, and later
  DMN input-contract work can reuse stable engine-owned placeholders
- one optional direct `variable` placeholder is preserved under each
  top-level `inputData`
- top-level `knowledgeSource` metadata is preserved so lint, adapter, and
  later authority-reference work can reuse stable engine-owned placeholders
- top-level `businessKnowledgeModel` metadata is preserved so lint,
  adapter, and later knowledge-contract work can reuse stable
  engine-owned placeholders
- one direct body `literalExpression` placeholder is preserved under each
  top-level `businessKnowledgeModel`
- top-level `decisionService` metadata is preserved so lint, adapter, and
  later service-contract work can reuse stable engine-owned placeholders
- direct decision-service references are preserved as href placeholders only;
  this includes direct `outputDecision`, `encapsulatedDecision`,
  `inputDecision`, and `inputData`
- top-level `organizationUnit` metadata is preserved so lint, adapter, and
  later governance-contract work can reuse stable engine-owned placeholders
- top-level `performanceIndicator` metadata is preserved so lint, adapter,
  and later monitoring-contract work can reuse stable engine-owned
  placeholders
- top-level `textAnnotation` metadata is preserved so lint, adapter, and
  later annotation-contract work can reuse stable engine-owned placeholders
- one direct nested text payload is preserved under each top-level
  `textAnnotation`
- top-level `association` metadata is preserved so lint, adapter, and later
  document-structure work can reuse stable engine-owned placeholders
- one direct `sourceRef` and one direct `targetRef` placeholder are
  preserved under each top-level `association`
- top-level `elementCollection` metadata is preserved so lint, adapter, and
  later structural-grouping work can reuse stable engine-owned
  placeholders
- top-level `group` metadata is preserved so lint, adapter, and later
  visual-artifact work can reuse stable engine-owned placeholders
- top-level `dmndi:DMNDI` metadata is preserved so lint, adapter, and later
  diagram-contract work can reuse stable engine-owned placeholders
- one direct nested `DMNDiagram` placeholder layer is preserved under each
  top-level `dmndi:DMNDI`
- direct nested `DMNShape` and `DMNEdge` placeholder metadata are preserved
  under each bounded `DMNDiagram`, limited to optional `id` plus
  `dmnElementRef`
- direct nested `DMNShape` placeholder metadata now also preserves optional
  `isListedInputData`
- direct nested `DMNShape` placeholder metadata now also preserves optional
  `isCollapsed`
- direct nested `DMNShape` placeholder metadata now also preserves one
  optional direct `dc:Bounds` placeholder bounded to one optional
  x/y/width/height contract
- direct nested `DMNEdge` placeholder metadata now also preserves one
  repeated direct `di:waypoint` placeholder list bounded to optional x/y
  pairs
- direct nested `DMNShape` and `DMNEdge` placeholder metadata now also
  preserve one optional direct `DMNLabel` placeholder bounded to one
  optional label id
- direct nested `DMNLabel` placeholder metadata now also preserves one
  optional direct `dc:Bounds` placeholder bounded to one optional
  x/y/width/height contract
- direct nested `DMNLabel` placeholder metadata now also preserves one
  optional direct text payload
- direct nested `DMNShape` placeholder metadata now also preserves one
  optional direct `DMNDecisionServiceDividerLine` placeholder bounded to
  one repeated direct `di:waypoint` placeholder list with optional x/y
  pairs
- decision-owned direct `invocation` metadata is preserved so lint, adapter,
  and later invocation-runtime work can reuse stable invoked-expression and
  binding placeholders without fabricating called function semantics
- decision-owned direct `functionDefinition` metadata is preserved so lint,
  adapter, and later function-runtime work can reuse stable parameter and body
  placeholders without fabricating function semantics

## Deferred

- item-definition resolution into executable clause typing
- broader input-data resolution into executable decision inputs beyond the
  current same-source alias-bind reuse
- recursive or arbitrary-depth item-component traversal
- recursive or broader variable/type-model traversal
- authority-reference resolution beyond the current bounded target counts
- business-context execution, thresholds, targets, or organization
  hierarchies
- annotation execution, interpretation, or routing semantics
- association graph resolution or executable routing semantics beyond the
  bounded direct direction/ref placeholder capture
- element-collection membership parsing beyond bounded id/name metadata
- group-to-DMNDI relationships or visual-layout interpretation
- `DMNShape` / `DMNEdge` metadata beyond optional `id`, `dmnElementRef`,
  direct `DMNShape.isListedInputData`, direct `DMNShape.isCollapsed`, one
  optional direct `dc:Bounds` placeholder under `DMNShape` with one
  optional x/y/width/height contract, one repeated direct `di:waypoint`
  placeholder list under `DMNEdge` with optional x/y pairs, and one
  optional direct `DMNLabel` placeholder with one optional label id plus
  one optional direct `dc:Bounds` placeholder plus one optional direct text
  payload, plus one optional direct `DMNDecisionServiceDividerLine`
  placeholder under `DMNShape` with one repeated direct `di:waypoint`
  placeholder list with optional x/y pairs
- broader DMNDI layout interpretation beyond the current direct
  `DMNDecisionServiceDividerLine` placeholder capture
- broader XML text capture beyond the current bounded `textAnnotation/text`
  seam
- business-knowledge-model body execution or evaluation
- decision-service reference resolution, output decision resolution, or
  execution
- broader requirement-reference href resolution or DRD dependency execution
  beyond the current same-source `requiredDecision` recursion and
  `requiredInput` alias-bind slice
- direct invocation execution, called-function resolution, or binding
  evaluation
- direct function-definition execution, function body evaluation, or
  parameter-binding evaluation
- broader DRD dependency execution
- broader FEEL or schema-validation completeness
