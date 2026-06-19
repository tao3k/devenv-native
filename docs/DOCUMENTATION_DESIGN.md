---
type: knowledge
metadata:
  title: "Documentation Design Specification (DFS-2026)"
---

# Documentation Design Specification (DFS-2026)

> **Authority:** CyberXiuXian Artisan workshop
> **Standard:** DFS-2026 (Filesystem-Based Documentation Hierarchy)  
> **Status:** Ratified

## 1. Purpose

The primary objective of this system is to eradicate "documentation entropy" and establish a cognitive asset management system with **industrial-grade precision**. Key goals include:

- **Cognitive Decoupling**: Physically isolating the _Why_ (Vision), _How_ (Core), _Rules_ (Dev), and _Truth_ (Features).
- **LLM Optimization**: Utilizing path-based numbering and a dedicated machine interface (`99_llm`) to maximize the signal-to-noise ratio for AI agent retrieval and reasoning.
- **Absolute Traceability**: Ensuring every technical claim (Feature) is backed by verifiable evidentiary archives.

## 2. Scope

This specification governs all Markdown, JSON, YAML, and associated asset files within the `docs/` directory. Any file violating the DFS-2026 path convention is considered "unclassified data" and must be remediated or deleted during the next audit cycle.

## 3. Structure

Documentation is organized into a **Layered Namespace** structure, where the filesystem path prefix defines the file's responsibility:

| Path Level       | Semantic Name       | Responsibility                                                                                                     | Audience     |
| :--------------- | :------------------ | :----------------------------------------------------------------------------------------------------------------- | :----------- |
| `00_vision/`     | **Vision Layer**    | High-level philosophy, system constitution, long-term strategy, and the Trinity Manifesto.                         | Human        |
| `01_core/`       | **Core Layer**      | Mathematical models, algorithmic specifications (Wendao/Memory/Omega/context injection), and technical blueprints. | LLM/Dev      |
| `02_dev/`        | **Dev Layer**       | Coding standards, workflow protocols, traceability policies, and developer handbooks.                              | Developer    |
| `03_features/`   | **Feature Layer**   | **[SSOT]** Single Source of Truth. Contains only 100% implemented and verified feature details.                    | LLM/Audit    |
| `04_chronicles/` | **Chronicle Layer** | Historical milestones, backlogs, and archived legacy plans.                                                        | Human        |
| `99_llm/`        | **Machine Layer**   | LLM-optimized bootstrap indices, schema mappings, and system state snapshots.                                      | **LLM Only** |

## 4. Content Requirements

Each layer must adhere to specific content standards:

- **01_core**: Must include mathematical derivations, Rust module mappings, and research paper alignment notes.
- **02_dev**: Must include concrete command examples (Justfile) and Nix environment specifications.
- **03_features**: Must explicitly link to verified integration test files in the `tests/` directory.
- **99_llm**: Must maintain high information density, eschewing conversational filler in favor of direct path-mapping and schema definitions.

## 5. Formatting Standards

- **Syntax**: Uniform use of GitHub Flavored Markdown (GFM).
- **Naming**: Root-level indices and manifestos use `SCREAMING_SNAKE_CASE.md`; detailed specs use `kebab-case.md`.
- **References**: Cross-layer references must use relative paths (e.g., `../../01_core/...`). Absolute paths are strictly prohibited to ensure portability.

## 6. Tooling

- **Parsing**: `comrak` (Rust) for AST extraction.
- **Indexing**: `xiuxian-wendao` for digitizing the directory into a Mixed Directed Graph.
- **Validation**: local Rust context-injection checks for XML tag integrity during context assembly.

## 7. Review Process

- **Standard**: Every document must pass the "Artisan Triple Audit": Logic Alignment, Research Verification, and Code Traceability.
- **Automation**: All `03_features` documents must eventually be verified by an automated `test-link-checker` to ensure referenced test files exist and pass.

## 8. Delivery & Version Control

- **Synchronization**: Documentation must be updated in tandem with code changes. Documentation lag is considered a high-priority technical debt.
- **Snapshots**: Upon reaching major milestones, a read-only snapshot of `99_llm/system_context.xml` is generated for Omega strategic auditing.
