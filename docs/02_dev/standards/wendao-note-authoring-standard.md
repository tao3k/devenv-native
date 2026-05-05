---
type: knowledge
title: "Wendao Note Authoring Standard"
category: "standards"
tags:
  - standards
  - wendao
  - link-graph
  - templates
saliency_base: 7.2
decay_rate: 0.03
---

# Wendao Note Authoring Standard

> Purpose: maximize `xiuxian-wendao` retrieval precision (graph + lexical) on Markdown corpora.

## 1. Scope

This standard applies to all Markdown notes intended for LinkGraph indexing:

- `docs/**/*.md`
- `assets/skills/**/SKILL.md`
- future notebook folders that are included by `link_graph.include_dirs`.

## 2. Hard Requirements

1. Every note must have one stable H1 title.
2. Every note must use deterministic slug-like filename (no random suffixes).
3. Every note must contain at least one explicit outbound link (except terminal archive notes).
4. Headings must be semantically meaningful; avoid placeholder headings like `Misc` or `Temp`.
5. Use English for technical content in repository-managed docs.

## 3. Frontmatter Contract (The ZK/Org-Roam Protocol)

Use this contract at the top of each note. This ensures unique identification consistent with Zettelkasten and Org-Roam principles.

```yaml
---
id: "YYYYMMDDHHMMSS" # Mandatory Unique ID (ZK/Org-Roam compatible)
title: "Human-readable title"
category: "architecture|reference|plans|standards|testing|how-to|explanation"
tags:
  - "domain-tag-1"
saliency_base: 5.0
decay_rate: 0.05
metadata:
  <extension_key>: "<extension value>"
---
```

The `metadata` mapping is optional extension space. Do not duplicate top-level
frontmatter fields such as `title`, `category`, `tags`, `saliency_base`, or
`decay_rate` under `metadata`.

## 4. Atomic Note Structure (Zettelkasten)

1. **One Concept, One Note**: Each note should focus on a single atomic concept or decision.
2. **Permanent vs. Ephemeral**:
   - **Daily Notes** (`$PRJ_CACHE_HOME/agent/GTD/DAILY_*.md`) serve as the "Roam-style" journal for raw input.
   - **Permanent Notes** (categorized core docs) are refined, alchemized knowledge.

## 5. Link and Lineage Rules

Use explicit links to improve graph traversal quality. Obsidian officially also
accepts bare `[[target]]`, heading links such as `[[target#Heading]]`, and
block links such as `[[target#^block-id]]`, but repository authoring remains
stricter than the raw parser compatibility surface:

- **Wiki links**: `[[target-note-stem|display-label]]` for concept relationships when you want repository-native wikilinks. The display label must be descriptive; do not repeat the raw target stem or heading fragment.
- **Markdown links**: `[display-label](target-note.md)` are equally valid when that form reads better.
- **Bi-directional Integrity**: Every note MUST reference its "Parent" or a Map of Content (MOC).
- **Backlink Section**: Notes should include a `## References` or `## Linked Notes` section at the bottom to maintain the physical trace of the knowledge web.

Recommended relation block:

```markdown
## Linked Notes

- Related: [[router|Router]]
- Depends on: [[vector-router-schema-contract|Vector Router Schema Contract]]
- Compared with: [[link-graph-vs-librarian|Link Graph vs Librarian]]
```

## 6. Retrieval Anchor Rules

To improve lexical matching and route recall:

1. Include exact key terms users are likely to query.
2. Keep one concise definition sentence near the top of each `##` section.
3. Avoid synonym-only wording; include canonical project term at least once.

## 7. Template Set

Use the template files under:

- `docs/standards/templates/wendao/concept-note.template.md`
- `docs/standards/templates/wendao/decision-note.template.md`
- `docs/standards/templates/wendao/moc-note.template.md`
- `docs/standards/templates/wendao/experiment-note.template.md`

## 8. Review Checklist

Before merging a new/updated note:

1. Frontmatter is valid and complete.
2. H1/H2 structure is stable and meaningful.
3. At least one outbound link exists.
4. Query anchors (primary terms) are present.
5. File path and stem are deterministic.
