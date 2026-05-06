# Standard: Link-Driven Metadata & Identity Positioning (V5.0)

We decouple the **Discovery** of a skill from the **Validation** of its content.

## 1. Skill Discovery (The Wendao Law)

A directory is recognized as a **Skill** if and only if it contains a physical **`SKILL.md`** file.

- **Physical Identifier**: `SKILL.md` serves as the anchor for the `wendao://skills/` namespace.
- **Trigger**: The presence of `SKILL.md` selects the parser-owned SKILL.md
  frontmatter contract in `xiuxian-wendao-parsers`.

## 2. Content Validation (The Parser Law)

Once discovered, Wendao linting enforces the following through the parser-owned
Markdown/frontmatter contract:

### 2.1 Default Standards (Optional but Recommended)

- **AUDIT.md**: Highly recommended for industrial traceability. If missing, a warning is emitted, but the skill is accepted.

### 2.2 Frontmatter and Metadata Boundary

Every `.md` file (Skill manifest or Persona) MUST keep canonical document
identity in top-level YAML frontmatter.

- **Core identity**: `title`, `kind` or type-equivalent fields, `category`,
  `tags`, provenance fields, and retrieval hints belong at top level.
- **metadata**: Optional extension space. Do not duplicate top-level identity
  fields such as `title` under `metadata`.
- **Failure**: malformed YAML or missing required top-level fields will trigger
  a load-blocker for that specific asset.
