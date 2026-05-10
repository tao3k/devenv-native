---
type: knowledge
title: "LLM Documentation Policy & Execution Standards"
category: "meta"
tags:
  - guidelines
  - standards
  - llm-policy
saliency_base: 10.0
decay_rate: 0.0
metadata:
  title: "LLM Documentation Policy & Execution Standards"
---

# LLM Documentation Policy & Execution Standards

> **MANDATORY POLICY FOR ALL LLM AGENTS:**
> This document defines the absolute standards for reading, creating, structuring, and updating documentation within the `xiuxian-artisan-workshop` repository.
> Do not use this file to document technical stack details (like TOML vs YAML); those belong in their respective component specs. This file governs the **Documentation Lifecycle**.

## 1. The Documentation Lifecycle & State Tracking

When an LLM agent receives a task to update progress or implement a feature, it must follow a strict documentation state-machine:

1. **Verify State (`docs/03_features/`)**: Before writing code, the LLM MUST read the corresponding feature plan to understand the current `Status` and the `Execution Backlog`.
2. **Execute & Test**: Perform the code modifications and verify them.
3. **Synchronize State (Audit Log)**: Immediately after code execution succeeds, the LLM MUST update the target feature document.
   - Mark the checklist item as `(Done)`.
   - Append a timestamped, concrete entry to the `Audit Log / Decision Journal` section detailing exactly what was changed and what evidence (tests) proves it.

**Rule of Thumb:** Code is not "Done" until the feature plan's checklist and audit log reflect the change.

## 2. Directory Structure & Separation of Concerns

LLM Agents must place new documentation in the mathematically correct directory. **Never create a monolithic "Big Ball of Mud" `SPEC.md`.**

- **`docs/01_core/` (Theory & Architecture):**
  - _Purpose:_ Why is the system built this way? What academic papers back this up? What are the interface contracts?
  - _Rule:_ If a core spec becomes too long, extract sub-systems into an `architecture/` sub-directory and use Markdown hyperlinking (e.g., `See [Sub-System](./architecture/sub-system.md)`).
- **`docs/03_features/` (Project Management & Status):**
  - _Purpose:_ Tracking execution backlogs, capability matrices (Done/Pending), and chronological audit logs.
  - _Rule:_ Do not pollute these files with deep code tutorials or philosophical architecture debates. Keep them as actionable checklists.
- **`docs/03_features/*_scenarios.md` (Use Cases):**
  - _Purpose:_ Mapping out specific, tangible user flows (e.g., "The Over-Ambitious Afternoon" debate).
  - _Rule:_ Scenarios must be separated from Feature Plans to ensure checklists remain readable.
- **`docs/testing/` (QA & Runbooks):**
  - _Purpose:_ Manual runbooks, CI gate explanations, and benchmark tracking.

## 3. Formatting & Content Rules

- **Frontmatter:** EVERY new markdown file MUST contain a YAML frontmatter block at the very top (including `title`, `category`, `tags`, `saliency_base`, and `decay_rate`) to ensure compatibility with our internal documentation parser.
- **Wendao Graph Linking (Wikilinks):** Obsidian officially supports `[[note]]`, `[[note|display]]`, `[[note#heading]]`, `[[note#heading|display]]`, `[[note#^block]]`, and standard Markdown links. Repository authoring is intentionally stricter: when referencing other internal documents within this repository, you MUST use either the explicit wikilink form `[[path/to/document.md|Document Title]]` or the Markdown link form `[Document Title](path/to/document.md)`. Bare wikilinks such as `[[path/to/document.md]]` are not allowed in repository Markdown. The display label must add real display meaning; do not mechanically repeat the raw target or addressed heading as `[[path/to/document.md|path/to/document.md]]` or `[[Note#Heading|Heading]]`. **DO NOT** mix wikilinks with Markdown link syntax such as `[[path/to/document.md]](Document Title)`, and do not reverse the alias order to `[[Document Title|path/to/document.md]]`. This keeps repository authoring aligned to one explicit contract for human and LLM readers while preserving link extraction for `xiuxian-wendao`.
- **Precision and Brevity:** Avoid conversational filler ("I will now update...", "Here is the document..."). State the invariant, present the evidence, and outline the architecture.
- **Traceability:** Whenever making a claim that a feature is working, the LLM MUST cite the exact test file path or script used as evidence.
- **Rust Testing Evidence:** When citing Rust test commands in documentation, audit logs, or feature plans, **ALWAYS** use `cargo nextest run` (e.g., `cargo nextest run -p xiuxian-wendao`). DO NOT use the legacy `cargo test` command.
- **Rust Validation Scope:** When citing executable Rust validation commands, prefer crate-scoped commands first (for example `cargo nextest run -p xiuxian-wendao`). Expand to cross-crate or workspace-wide runs only when required.

## 4. Persona Integrity

When an LLM is instructed to adopt a specific persona (e.g., `Strict Architecture Auditor`), it must maintain the tone in its conversational output, but ensure the actual markdown documentation it generates or modifies remains objective, professional, highly structured, and free of conversational role-play artifacts.
