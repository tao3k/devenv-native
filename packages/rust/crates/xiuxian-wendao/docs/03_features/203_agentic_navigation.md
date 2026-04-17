# Agentic Navigation (wendao.agentic_nav)

:PROPERTIES:
:ID: feat-agentic-nav
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, search, discovery, agentic
:STATUS: STABLE
:VERSION: 2.4
:END:

## Overview

`wendao.agentic_nav` is a reasoning-driven discovery tool that acts as a "Structured GPS" for Agents. It bridges the gap between neural vector search and symbolic AST validation.

## Core Capabilities

1. **Skeleton Re-ranking**: Automatically prioritizes search hits that match the document's structural skeleton.
2. **Navigation Hints**: Returns a `<navigation_hint>` for each candidate, explaining its structural role (e.g., "Top-level section", "Deeply nested implementation details").
3. **Identity Verification**: Checks if the target `:ID:` is still valid in the live AST before returning it.

## Output Schema (LLM-Native)

The tool produces a structured XML response optimized for Agent parsing:

```xml
<agentic_nav_result>
  <query>refactor storage</query>
  <candidates>
    <candidate>
      <doc_id>README.md</doc_id>
      <anchor_id>#arch-v1</anchor_id>
      <navigation_hint>Top-level section - good entry point.</navigation_hint>
      <structural_path>
        <segment>Architecture</segment>
        <segment>Storage</segment>
      </structural_path>
    </candidate>
  </candidates>
</agentic_nav_result>
```

## Studio Reader Payload

The Studio markdown-analysis payload now carries a backend-owned
`documentMetadata` block for reader-facing DeepWiki identity:

1. `title`, `tags`, and `docType` come from parser-owned document metadata,
   with docs-kernel `:PROPERTIES:` acting as the bounded fallback for document
   tags and type.
2. `parent` comes from the explicit docs-kernel `:PARENT:` declaration rather
   than frontend frontmatter heuristics.
3. `outgoingLinks` materialize explicit property-drawer relations plus
   docs-kernel `:RELATIONS: :LINKS:` rows.
4. `explicitBacklinks` come from the link-graph reverse index when the Studio
   runtime has a live graph index, with parser-backed reconstruction used to
   preserve scoped `targetAddress` values such as `#Heading` and `#^block`.
   `backlinks` remains the compatibility alias for the same explicit lane.

This keeps the reader as a projection over parser and index truth sources
instead of a second wikilink parser.

:RELATIONS:
:LINKS: [[01_core/101_triple_a_protocol|Triple-A Addressing Protocol]], [[05_research/302_search_as_reasoning|Search-as-Reasoning: Autonomous Search in Structured State Spaces]]
:END:
