# DMN Information Requirements

This module records the bounded `qianji-bpmn-engine` executable contract for
direct DMN `informationRequirement` references.

## Current Support

- executable DMN decision definitions preserve direct `requiredInput` and
  `requiredDecision` href placeholders in source order
- the preserved executable contract is exposed as
  `DmnInformationRequirementReference`
- parser-owned bundle loading now also derives one bounded package-level
  `inputData` registry from top-level DMN `inputData` metadata for executable
  runtime use
- one bounded local runtime path now resolves direct same-source
  `requiredDecision` dependencies before evaluating the current decision body
- one bounded same-source `requiredInput` path now aliases a caller-supplied
  top-level `inputData.name` value into the nested `variable.name` expected by
  the current decision when both names are explicit in the source metadata

## Runtime Boundary

The current evaluator still does not execute broader DRD graphs. It only
supports direct same-source `requiredDecision` recursion for locally registered
decisions plus one bounded same-source `requiredInput` alias bind for locally
registered `inputData` definitions. Cycles are rejected explicitly, and
missing or non-local href targets are rejected explicitly. The caller must
still supply the input object under the top-level `inputData.name`;
`knowledgeRequirement`, `authorityRequirement`, `decisionService`, import
resolution, output-side alias persistence, and broader input-data mapping
remain deferred.

## Guidance

When widening beyond this subset, keep using the parsed
`information_requirements` contract instead of re-reading document snapshots or
re-parsing XML. Extend resolution explicitly from the current same-source
`requiredDecision` recursion plus bounded `requiredInput` alias-bind path
rather than silently folding in broader DRD planning or generalized input-data
remapping.
