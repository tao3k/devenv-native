---
kind: SKILL.md
type: skill
title: "Wendao Documentation Guardian"
category: "documentation"
tags:
  - wendao
  - documentation
  - governance
name: wendao
description: Internal guardian skill for creating and editing Wendao DocOS documentation.
author: xiuxian-artisan-workshop
date: 2026-04-26T09:30-07:00
metadata:
  retrieval:
    saliency_base: 5.5
    decay_rate: 0.05
  version: "2.0.0"
  source: "https://github.com/tao3k/xiuxian-artisan-workshop/tree/main/internal_skills/wendao"
  routing_keywords:
    - wendao
    - documentation
    - docos
    - project anchor
---

# Wendao Documentation Guardian Skill

:PROPERTIES:
:ID: skill-wendao-guardian
:TYPE: METASKILL
:STATUS: ACTIVE
:END:

## 🎯 Objective

Act as the authoritative guardian for creating and editing Wendao DocOS documentation. Enforce the Project AnchoR v2.0 standard to ensure AST-level compatibility and semantic integrity.

## 📜 Mandatory Template (AnchoR Standard)

All documents created under this skill MUST follow this structure:

1. **Identity Header**:

   ```markdown
   # Title

   :PROPERTIES:
   :ID: <semantic-slug>
   :PARENT: [[<parent-file>]]
   :TAGS: <tags>
   :STATUS: <DRAFT|STABLE|DEPRECATED>
   :END:
   ```

2. **Semantic Relations**:
   Replace "SEE ALSO" or "Footnotes" with a structured RELATIONS drawer:

   ```markdown
   :RELATIONS:
   :LINKS: [[link1]], [[link2]]
   :END:
   ```

3. **Block IDs**:
   For critical paragraphs or code blocks, append a block anchor if missing: `^id-123`.

## 🛠️ Constraints for LLM

- **No Line Numbers**: Never refer to line numbers in documentation. Use `#id` or `/path`.
- **Org-style Only**: Use `:KEY: VALUE` for all metadata.
- **Atomic Notes**: One concept per file (Zettelkasten).
- **English Context**: Body must be in English for international technical consistency.

## 🔍 Validation Logic

Before outputting, check:

- Does it have a `:PROPERTIES:` drawer with a unique `:ID:`?
- Is the `:PARENT:` correctly linked?
- Are all links using the `[[filename]]` ZK syntax?

---

:FOOTER:
:STANDARDS_VERSION: 2.0
:AUDITOR: auditor_neuron
:END:
