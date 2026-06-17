# Code Observation (:OBSERVE:)

:PROPERTIES:
:ID: feat-code-observation
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, language-provider, non-invasive
:STATUS: STABLE
:VERSION: 2.7
:END:

## Overview

Non-Invasive Code Observation allows Wendao documents to "bind" to source code blocks using provider-owned structural patterns. This keeps documentation and implementation semantically aligned without polluting the source code.

## Core Mechanism

- **Engine**: Owned by the external language-provider boundary; local structural parser crates are retired from Wendao.
- **Trigger**: Property drawer attribute `:OBSERVE: lang:<language> "<pattern>"`.
- **Validation**: `wendao audit` preserves the `:OBSERVE:` contract and delegates structural validation to the language-provider integration when that provider is available.

## Multi-Observation Support

A single section can track multiple code entities:

```markdown
:PROPERTIES:
:OBSERVE_1: lang:rust "struct $NAME { $$$FIELDS }"
:OBSERVE_2: lang:rust "impl $NAME { $$$METHODS }"
:END:
```

## LLM-Friendly Diagnostics

If a pattern is syntactically invalid, the auditor provides granular feedback:

```xml
<issue code="ERR_INVALID_OBSERVER_SYNTAX" severity="ERROR">
  <message>Invalid pattern: Unexpected EOF</message>
  <suggestion>Ensure the provider pattern contains a complete structural unit.</suggestion>
</issue>
```

## Studio Gateway Status (2026-03-19)

- `:OBSERVE:` metadata now participates in Wendao Studio definition resolution through backend-shared `lang:` and `scope:` hints.
- Studio graph symbol navigation now resolves through the same backend resolver used by `/api/search/definition`.
- Studio search payloads now ship display-ready `navigationTarget` metadata so Qianji Studio does not need to infer code-vs-doc routing from Markdown observation payloads or raw paths.
- Studio graph payloads now also ship `navigationTarget` on live `/graph/neighbors` Flight responses, replacing the older split `navigationPath`/`line`/`column` shape with the same backend-owned navigation contract used by search.
- Bilink graph-miss fallback resolution now also lives in the gateway through `/vfs/resolve` on Flight, so semantic path normalization no longer depends on frontend candidate expansion.
- Studio VFS scan payloads now expose `project_root` and `project_dirs` alongside `project_name` and `root_label`, so FileTree grouping and hover provenance remain gateway-owned.
- Qianji Studio still keeps minimal search and graph fallbacks when transitional payloads omit top-level `navigationTarget`, but the intended contract remains backend-owned navigation metadata and is now exercised by the live gateway test lane.

:RELATIONS:
:LINKS: [[01_core/101_triple_a_protocol|Triple-A Addressing Protocol]], [[03_features/205_semantic_auditor|Semantic Auditor (wendao audit)]]
:END:
