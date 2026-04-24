---
type: knowledge
title: "ADR-008: Declarative Skill Structure and Validation"
status: "Superseded"
date: "2026-02-28"
category: "architecture"
tags:
  - zhenfa
  - scanner
  - validation
  - config
metadata:
  title: "ADR-008: Declarative Skill Structure and Validation"
---

# ADR-008: Declarative Skill Structure and Validation

Supersession note: the standalone skills crate described here has been retired.
`xiuxian-wendao-parsers` now owns SKILL.md/frontmatter parsing and lint
contracts, while Wendao owns runtime skill VFS inventory, internal alias
contracts, and schema resources.

## 1. Context and Problem Statement

The physical directory structure of an **Agent Skill** (e.g., `SKILL.md`,
`scripts/`, `references/`) was previously defined by hardcoded defaults in the
retired skills crate or scattered in legacy `settings.yaml` files.

This creates several issues:

- **Inflexibility**: Users cannot easily define or enforce their own organizational standards for skills without changing code.
- **Verification Gap**: There is no centralized, single-source-of-truth validation engine to ensure that new skills comply with our quality standards (e.g., ensuring all docs are in `references/`).
- **Legacy Debt**: The project is transitioning to a TOML-first configuration paradigm, rendering the YAML-based `settings.yaml` obsolete.

## 2. Decision

We originally planned to transition to a **Declarative Skill Structure**
governed by a centralized TOML configuration. The active architecture has since
superseded that plan: parser-owned Markdown contracts validate SKILL.md
frontmatter, and Wendao-owned runtime inventory consumes the parsed contract.

### 2.1 Centralized Configuration

A crate-local configuration file was proposed as the "Constitution" for skill
organization. The current implementation does not keep that file in a
standalone skills crate; active rules live in parser/Wendao-owned surfaces. The
conceptual rule set was:

- `required`: Files/Directories that MUST exist for a skill to be considered valid.
- `default`: The canonical layout used when scaffolding a new skill.
- `validation`: Semantic rules, such as prohibiting logic in `SKILL.md` or enforcing the `references/` hierarchy.

### 2.2 Configuration-Driven Runtime

The current runtime should keep structural defaults in its owning crate rather
than reintroducing a shared skills package.

### 2.3 Extensibility

Users can override or extend these rules in their local `xiuxian.toml` under the `[skills.architecture]` key, allowing for domain-specific constraints (e.g., "all research skills must contain a `papers/` folder").

## 3. Technical Design

### 3.1 The `skills.toml` Schema (Conceptual)

```toml
[architecture]
required = [
    { path = "SKILL.md", description = "Skill manifest" }
]
default = [
    { path = "scripts/", item_type = "dir" },
    { path = "references/", item_type = "dir" }
]

[validation]
strict_mode = true
enforce_references_folder = true
```

### 3.2 Integrated Validation Flow

When the `Wendao` indexer or the `Agent` boots up:

1. Load `skills.toml`.
2. `SkillScanner` iterates through `assets/skills/*`.
3. For each folder, `scanner.validate(path, config)` is called.
4. If validation fails, the skill is ignored or flagged with a warning, preventing the injection of malformed tools into the LLM context.

### 3.3 Internalized Resource Layout (Crate Self-Containment)

To eliminate path ambiguity and ensure portability, all system-level "Constitutions" (like `skills.toml`) will be stored within the **internal `resources/` directory** of the managing crate.

- **Standard Paths**:
  - `packages/rust/crates/xiuxian-wendao/resources/...`
  - `packages/rust/crates/xiuxian-daochang/resources/...`
- **Mechanism**: The crate uses a direct, non-escaping `include_str!("resources/config/skills.toml")`.
- **Portability**: This architecture keeps validation laws embedded in the
  owning runtime crate, requiring no external files to perform a baseline
  structural check.

## 4. Consequences

### Positive

- **Architectural Guardrails**: Forces all AI and human developers to follow the same organizational pattern.
- **Scalability**: New file types (e.g., `assets/`, `config/`) can be added to the standard without touching core logic.
- **Improved DX**: Clear, error-driven feedback when a developer misplaces a file.

### Negative

- **Startup Overhead**: Minor latency increase during boot to perform the structure scan (mitigated by Rust's speed and optional caching).

## 5. Implementation Plan

1.  Keep SKILL.md parsing and lint entrypoints in `xiuxian-wendao-parsers`.
2.  Keep runtime skill VFS inventory and schema resources in `xiuxian-wendao`.
3.  Avoid introducing a replacement shared skills crate.
