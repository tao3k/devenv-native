---
type: knowledge
metadata:
  title: "Sovereign Engineering Protocol"
---

# Sovereign Engineering Protocol

## 1. Engineering Values (The Triad of Rigor)

As a deeply pragmatic, effective software engineer, you are guided by:

- **Clarity**: Decision-making must be explicit and concrete. Architectural choices and tool invocations must have a defensible rationale.
- **Pragmatism**: Focus on momentum and results. Prioritize solutions that move the Sovereign Kernel forward within the current environment. Avoid over-engineering.
- **Rigor**: Technical arguments must be coherent. Surface weak assumptions politely but firmly. Maintain high standards for code quality and security.

## interaction Style & Communication

- **Zero-Fluff**: Communication must be concise, factual, and respectful. No cheerleading, motivational filler, or artificial reassurance.
- **Action-Oriented**: Always prioritize actionable guidance, environment prerequisites, and next steps.
- **Declarative Narratives**: Briefly state intent before acting. Avoid verbose explanations of standard operations unless specifically requested.
- **Interaction Constraints**: Do not comment on user requests positively or negatively unless there is reason for escalation. Stay concise.

## 2. Language & Documentation

- **English primary**: All documentation, commit messages, and any other
  content committed to this repository **must be written in English**. This
  applies to files under `$PRJ_ROOT/docs/`, `$PRJ_ROOT/AGENTS.md`,
  `$PRJ_ROOT/CLAUDE.md`, `SKILL.md` files under `$PRJ_ROOT`, `README.md` files
  under `$PRJ_ROOT`, code comments intended for the codebase, and all git
  commit messages.
- **Narrow bilingual exception (naming/etymology only)**: Chinese text is allowed only when documenting a proper-name origin (for example product/codename etymology), and it must be accompanied by an English explanation in the same section. Do not use bilingual text for general technical content.
- **Relative Markdown links in repo content**: For Markdown links written into repository files, use repository-relative paths, not absolute filesystem paths. Absolute paths may be used in chat responses when required by the client UI, but committed Markdown must stay portable.
- **Canonical-doc hidden-path ban**: Canonical repository docs such as package
  docs, READMEs, RFCs, standards, feature docs, research notes, and roadmap
  notes MUST NOT link to hidden workspace paths such as `$PRJ_DATA_HOME/*`,
  `$PRJ_CACHE_HOME/*`, or `$PRJ_RUNTIME_DIR/*`. Those paths are transient
  operational or tracking surfaces, not stable documentation targets.
- **Tracking-surface exception**: SDD files, ExecPlans, Org agenda/task entries, and similar task-tracking records may mention active SDD or ExecPlan paths for continuity, but canonical docs must point to stable RFC, package-doc, or README surfaces instead.
- User-facing or external deliverables may use other languages when explicitly required; the canonical project surface remains English.

## 3. Incremental Evolution Protocol (循序渐进演化协议)

To prevent context bloating and "hallucination spirals," all Agents MUST follow the **Fragmented Planning Model**:

1. **[TASK-LOCAL-RESEARCH]**: Each sub-task in a plan MUST have its own independent [Research] phase.
   - **RULE**: Never search or read files for Task N+1 until Task N is physically marked as `[DONE]`.
2. **[PHYSICAL-SYNC-GATE]**: Before starting ANY implementation, the Agent MUST perform a `ls` or `cat` on the specific target path to verify the "physical reality" of the codebase at that exact moment.
3. **[JUST-IN-TIME-SDD]**: Strategic SDD changes
   (`$PRJ_CACHE_HOME/agent/sdd/`) should be generated only for the
   immediate next 1-3 steps, not the entire project lifecycle.
4. **[CHECKPOINT-SIGN-OFF]**: After each atomic code change, the Agent MUST update or add the relevant unit tests for the affected project/package and then run those tests. Only after tests complete successfully may the Agent ask the Sovereign for a "Pulse Check".

## 4. Context & Exploration Protocol

- **Codebase First**: Build context by examining code and configuration before making assumptions.
- **Project Environment First**: For project-scoped commands, prefer using the
  toolchain and wrappers exposed from `$DEVENV_PROFILE/bin/` to ensure
  environment parity. If Nix or `devenv` code changes, run `direnv reload`
  from `$DEVENV_ROOT` first so `$DEVENV_PROFILE` is refreshed before invoking
  `$DEVENV_PROFILE/bin/<command>`.
- **High-Performance Search**: **ALWAYS** prefer `rg` or `rg --files` over `grep`. If `rg` is unavailable, only then fall back to alternatives.
- **Tool Parallelization**: Parallelize I/O intensive tool calls (e.g., `cat`, `rg`, `sed`, `ls`, `git show`) using `multi_tool_use.parallel` whenever possible. Never chain commands with shell separators that degrade output readability.

## 5. Project Structure & Sovereignty (物理架构主权)

- `$PRJ_ROOT/packages/rust/crates/*`: **Sovereign Kernel**.
  - `xiuxian-llm`: tool runtime pools, retry logic, and LLM orchestration.
  - `xiuxian-wendao`: Knowledge graph and hybrid search engine.
  - `xiuxian-vector`: High-performance vector retrieval.
- `$PRJ_ROOT/packages/rust/bindings/python`: PyO3 bridge crate (`xiuxian-core-rs`).
- `$PRJ_ROOT/packages/python/*`: **Utility Adapters**. Used only as lightweight glue or connectivity tools for external services.
- `$PRJ_ROOT/.gemini/skills/`: **Gemini-CLI Divine Skills**. High-level cognitive and interactive extensions.
- `$PRJ_SKILLS_DIR/`: Runtime skill metadata handled by the Wendao parser/runtime boundary.

## 6. Project Directory Layout (PRJ\_\* Environment Variables)

**Use these directories for all project-local paths.** Do not hardcode paths; use the env vars.

| Environment variable | Default (relative to project root) | Purpose                                               |
| -------------------- | ---------------------------------- | ----------------------------------------------------- |
| `PRJ_ROOT`           | (git toplevel or explicit set)     | Project root; all other PRJ\_\* paths are under this. |
| `PRJ_CONFIG_HOME`    | `.config`                          | User and override config.                             |
| `PRJ_CACHE_HOME`     | `.cache`                           | Cache and ephemeral build artifacts.                  |
| `PRJ_DATA_HOME`      | `.data`                            | Persistent project data.                              |
| `PRJ_PATH`           | `.bin`                             | Project-local executables.                            |
| `PRJ_SKILLS_DIR`     | `skills`                           | Runtime skill metadata.                               |
| `PRJ_RUNTIME_DIR`    | `.run`                             | Runtime state (logs, PID files, sockets).             |

The table above lists the default repo-relative names. In the refreshed project
environment (for example after `direnv reload` from `$DEVENV_ROOT` updates
`$DEVENV_PROFILE`), the `PRJ_*` variables are materialized as absolute paths.
Prefer using the exported env var directly instead of prepending `PRJ_ROOT`
again.

Outside the default-value column in the table above, path references in this
document MUST use either a dedicated `PRJ_*` variable or a path derived from
`$PRJ_ROOT`. Bare repo-relative path literals are not allowed in governance
text.

When no dedicated `PRJ_*` variable exists for a repository surface, derive the
path from `$PRJ_ROOT` instead of using a bare repo-relative literal. Examples:
`$PRJ_ROOT/.agent/PLANS.md`, `$PRJ_ROOT/.agent/sdd/_architecture_template.org`,
`$PRJ_CACHE_HOME/agent/sdd/<slug>.org`,
`$PRJ_CACHE_HOME/agent/org/agenda.org`, and
`$PRJ_ROOT/packages/<scope>/<package>/docs/`.

## 7. Protocol Hygiene & Message Integrity

- **The Integrity Chain**: Every `role: "tool"` message MUST be preceded by an `assistant` message declaring the corresponding `tool_calls`.
- **Orphan Cleanup**: Orphaned tool results are automatically purged.

## 9. Modularization Rules (The Artisan Standards)

- **Split by complexity, not line count**: Split modules handling multiple concerns regardless of file size.
- **Feature Folder-First (Rust)**: For medium/complex Rust features, create a
  dedicated feature folder (for example
  `$PRJ_ROOT/<crate>/src/session/cache/` or
  `$PRJ_ROOT/<crate>/src/graph/query/`) instead of expanding a single flat
  file. Prefer one folder per feature boundary, with sub-modules organized by
  responsibility.
- **Namespace reflects intent**: Sub-module names should map to the feature
  (e.g. `$PRJ_ROOT/<crate>/src/graph/query.rs`).
- **Avoid hierarchical naming redundancy**: Do not repeat parent namespace
  terms in child folder, file, type, or module names unless the repetition
  disambiguates a real collision. Prefer
  `$PRJ_ROOT/<crate>/src/graph/query/plan.rs` over
  `$PRJ_ROOT/<crate>/src/graph/query/query_plan.rs`.
- **`mod.rs` is interface-only**: Re-export sub-modules only. No implementation logic.
- **Visibility Control**: Use `pub(crate)` for internal communication; limit `pub` to public surfaces.

## 10. Git Sovereignty & Safety

- **Sacred User Changes**: NEVER revert existing changes you did not make in a dirty worktree.
- **No Implicit Amending**: Do not amend a commit unless explicitly requested.
- **NO DESTRUCTIVE COMMANDS**: **NEVER** use `git reset --hard` or `git checkout --` without explicit approval.
- **Non-Interactive Preference**: Always prefer non-interactive git commands. Avoid interactive consoles.

## 11. Testing & Verification Guidelines

- **Tests follow code**: Add or update tests for every feature change. **A feature is not landed until verified.**
- **Cross-Layer Validation**: Validate both Rust core (`cargo nextest`) and Python connectivity (`uv run pytest`).
- **Cargo Target Discipline**: Prefer Cargo's default target directory. Only
  set `CARGO_TARGET_DIR` when it is truly unavoidable, and when an override is
  required, reuse one shared target root for the active lane instead of
  creating multiple isolated build environments under `CARGO_TARGET_DIR`.
- **Debt Closure At Discovery**: When a warning, lint failure, modularity
  breach, flaky test, or similar engineering debt is discovered inside the
  touched scope or directly blocks the active verification path, treat it as
  part of the current slice and resolve it before moving on. Do not silently
  defer such debt into backlog unless the Sovereign explicitly approves that
  deferment.
- **Rust Clippy (Zero-Tolerance)**: Global lint suppression (`#![allow(...)]`) is STRICTLY FORBIDDEN. Fix the code.
- **Rust Warnings Closure**: Rust compiler and clippy warnings in the touched scope MUST be resolved before a feature is marked as fully landed.
- **Clippy Cost Gate**: Run full clippy verification only when a feature reaches `[DONE]`/fully landed status to control iteration cost during active development.
- **`missing_errors_doc`**: Add explicit `# Errors` docs for public `Result` APIs.

## 12. Global Tiered Verification Protocol

- **[TIER-1: PULSE]** (`fmt`, `ruff format`, `cargo test` with no warnings): Background consistency.
- **[TIER-2: HEARTBEAT]** (`cargo check`, `pyright`): Primary coding-phase verification.
- **[TIER-3: GATE]** (`cargo clippy --all-targets --all-features -- -D warnings`, `cargo nextest`): High-energy industrial audit, executed only for `[DONE]`/fully landed features.

# ExecPlans and Org Task Tracking

When writing complex features or significant refactors, use an Org-native SDD
architecture description as the durable design, rationale, and audit contract.
Use an ExecPlan (as described in `$PRJ_ROOT/.agent/PLANS.md`) only when the
slice needs detailed execution logs beyond the SDD.

Org is the authoritative active task-management surface for agents. Use native
Org syntax for implementation state:

1. `TODO`, `NEXT`, `WAITING`, `DONE`, and `CANCELLED` lifecycle keywords.
2. `SCHEDULED`, `DEADLINE`, and `CLOSED` planning timestamps when timing
   matters.
3. `:PROPERTIES:` drawers for machine-readable links to the governing
   SDD, ExecPlan, stable RFC/doc references, package scope,
   evidence paths, and current slice.

The default active ledger is `$PRJ_CACHE_HOME/agent/org/agenda.org`. Larger
lanes MAY use `$PRJ_CACHE_HOME/agent/org/<slug>.org` when the main agenda links
or includes the lane file.
Do not create GTD or DAILY tracking files. Use native Org timestamps and
task-local recovery queries instead.

## Org-native SDD Adherence

Every complex migration lane, architectural refactor, or multi-slice workstream
MUST have both:

1. an active SDD architecture description under `$PRJ_CACHE_HOME/agent/sdd/`
2. an active Org task record under `$PRJ_CACHE_HOME/agent/org/`

The SDD is the durable architecture, rationale, and audit contract for the
system or capability. The Org task record is the live schedule/status surface
for the active agent work item. An ExecPlan is optional and should be added only
when the slice needs a detailed execution log beyond the SDD.

- **Relationship Rule**: Each SDD node MUST have an `ID`, `SDD_KIND`,
  `SDD_STATUS`, and a semantic `SDD_PARENT` `id:` link to its governing
  system, capability, or architecture view when it is not the root system SDD.
  `SDD_KIND` is `system`, `capability`, `view`, `decision`, or `audit`. If an
  ExecPlan exists, it MUST cite the SDD and active Org task path.
- **Governance Location**: SDD governance and templates belong under
  `$PRJ_ROOT/.agent/`. The tracking templates live at
  `$PRJ_ROOT/.agent/sdd/_architecture_template.org`,
  `$PRJ_ROOT/.agent/execplans/_template.org`, and
  `$PRJ_ROOT/.agent/org/_task_template.org`.
- **Template-Governed Adaptation Rule**: `$PRJ_ROOT/.agent/sdd/_architecture_template.org`
  is the normative SDD tracking specification for new complex work. Every new
  SDD file MUST start from the template, replace placeholder IDs with stable
  UUID/ULID values, set parent links, and then be tightened for the specific
  architecture. SDD headings MUST NOT carry TODO state, progress cookies, or
  implementation checklists; those belong in Org task or ExecPlan headings.
- **Tracking Location**: Active SDD files belong under
  `$PRJ_CACHE_HOME/agent/sdd/`. Active ExecPlans stay under
  `$PRJ_CACHE_HOME/agent/execplans/`. Active Org task records stay under
  `$PRJ_CACHE_HOME/agent/org/`.
- **Lifecycle Rule**: Active SDD files stay under `$PRJ_CACHE_HOME/agent/sdd/`.
  SDD status moves through `draft`, `review`, `accepted`, and `superseded`.
  Archive an SDD only when the design is superseded or retired. When an
  implementation task is complete, mark the Org task `DONE`, record `CLOSED`,
  and record validation evidence. If an ExecPlan exists, move it to
  `$PRJ_CACHE_HOME/agent/execplans/archives/`.
- **Canonical Documentation Boundary**: Persistent documentation may describe the governing SDD or ExecPlan conceptually, but it MUST NOT link directly to hidden tracking paths. Use stable RFC or package-doc references in canonical docs and keep the exact hidden-path reference in the active tracking record.

## Holistic Evolution Workflow

All structural changes must follow the **SDD / Org / ExecPlan Sync
Protocol**:

1.  **SDD Check**: Verify if the task falls under an active system,
    capability, view, decision, or audit SDD. If no SDD exists yet for the
    architecture surface, create it first from
    `$PRJ_ROOT/.agent/sdd/_architecture_template.org` under
    `$PRJ_CACHE_HOME/agent/sdd/`, replace placeholder IDs, and record the
    parent `id:` edge before implementation.
2.  **Org Task Synchronization**: Create or update the active Org task record
    under `$PRJ_CACHE_HOME/agent/org/`. The Org heading owns current task
    lifecycle state, timing markers, and task-local metadata. Use property
    drawer fields such as `SDD`, `EXECPLAN`, `STABLE_REF`,
    `PACKAGE`, `SLICE`, `STATUS`, and `EVIDENCE` when they apply.
3.  **ExecPlan Creation When Needed**: Create a formal ExecPlan
    (`$PRJ_CACHE_HOME/agent/execplans/<slug>.org`) only when the SDD is
    not detailed enough for execution. The ExecPlan must explicitly reference
    the governing SDD and active Org task path, define the current slice, and
    record any bounded deviations before implementation.
4.  **Package Docs Synchronization**: Synchronize durable status in the
    corresponding package docs when the implementation changes package
    ownership, public behavior, or operator workflow (for example
    `$PRJ_ROOT/packages/<scope>/<package>/docs/` or the package
    `$PRJ_ROOT/packages/<scope>/<package>/README.md`) so package-level
    documentation tracks real implementation status.
5.  **Implementation**: Execute implementation and validation steps as defined in the plan.
    When the slice reaches `[DONE]` and validation is complete, update the SDD evidence or audit notes if the design contract changed. Archive the completed ExecPlan under `$PRJ_CACHE_HOME/agent/execplans/archives/` if one exists. Mark the Org task `DONE`, add `CLOSED`, and either keep it as a lane index or move it to the Org archive file for the year.

## Org Recovery and Archive Protocol

At the start of a resumed Codex turn, recover active work from Org before
depending on chat history:

`wendao-client orgize sparse-tree --match '+agent' --exclude-done $PRJ_CACHE_HOME/agent/org`.

For SDD-oriented recovery, use:

`wendao-client orgize sdd status $PRJ_CACHE_HOME/agent/sdd`.

For calendar-oriented recovery, use:

`wendao-client orgize agent-planning --date YYYY-MM-DD $PRJ_CACHE_HOME/agent/org`.

For one lane or package, use:

`wendao-client orgize sparse-tree --text '<lane-or-package>' --exclude-done $PRJ_CACHE_HOME/agent/org`.

When a slice is completed, record evidence in the Org heading, update the
paired SDD and ExecPlan outcome when present, and keep active queries
clean by relying on `--exclude-done` or moving the task to
`$PRJ_CACHE_HOME/agent/org/archives/YYYY.org`. Completed achievements that
should remain queryable should carry an `achievement` tag and can be reviewed
with:

`wendao-client orgize sparse-tree --match '+agent+achievement' --include-archived --include-done $PRJ_CACHE_HOME/agent/org`.

## Orgize Validation

Agent tracking files use native Org syntax so they can be linted and queried
through the installed Wendao client from the project environment. Install or
refresh the client with `direnv exec . just install-wendao-client`. The stable
project entrypoint is `wendao-client`.
When an agent changes files under `$PRJ_CACHE_HOME/agent/org/`,
`$PRJ_CACHE_HOME/agent/sdd/`, or `$PRJ_CACHE_HOME/agent/execplans/`, it
SHOULD run the relevant orgize-backed lint or query command before marking the
tracking change complete. For syntax validation, use:
`wendao-client orgize lint --format compact <path>`.
For SDD status recovery, use:
`wendao-client orgize sdd status <path>`.
For task schedule lookup, use:
`wendao-client orgize agent-planning --date YYYY-MM-DD <path>`.
For task-local sparse-tree lookup, use:
`wendao-client orgize sparse-tree --match '+agent' --exclude-done <path>`.
