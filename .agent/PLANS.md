# Execution Plan Policy

This repository uses ExecPlans for work that is large, uncertain, or cross
cutting. Active agent task state, scheduling, deadlines, and implementation
continuity are tracked in Org files under `$PRJ_CACHE_HOME/agent/org/`.
This repository uses Org as the agent tracking surface.

## Blueprint Adherence (Mandatory)

If a task falls under the scope of an existing strategic blueprint (located in `$PRJ_CACHE_HOME/agent/blueprints/`, repo-relative default `.cache/agent/blueprints/`), the ExecPlan MUST:

A blueprint is the durable architectural contract for a workstream. The ExecPlan is the task-local execution record for one bounded implementation slice under that contract.

1.  **Reference the Blueprint**: Explicitly link to the relevant blueprint file.
2.  **Strict Adherence**: The plan's architectural decisions, data models, and protocols must be derived directly from the blueprint.
3.  **Audit Alignment**: The `Reflection and Quality Audit` section must explicitly state how the implementation complies with the blueprint's mandates.
4.  **Org Alignment**: The plan must cite the active Org task record that owns
    current lifecycle state and timing markers for the slice.
5.  **No-Blueprint Case**: If no blueprint applies, state that explicitly in the plan and record why the task is outside blueprint governance.
6.  **Archive Discipline**: Keep active blueprints under `$PRJ_CACHE_HOME/agent/blueprints/` and move them to `$PRJ_CACHE_HOME/agent/blueprints/archives/` only when the governed workstream is complete. Keep active ExecPlans under `$PRJ_CACHE_HOME/agent/execplans/` and move them to `$PRJ_CACHE_HOME/agent/execplans/archives/` once the slice is DONE and validated. Mark the Org task `DONE`, record `CLOSED`, and either keep it as a lane index or move it to `$PRJ_CACHE_HOME/agent/org/archives/YYYY.org`.

Deviations from a blueprint are only allowed if explicitly requested by the Sovereign or if the blueprint itself is updated first.

## Plan First Gate (Mandatory)

Before any file reads, searches, or command execution (including tests), you must present a plan.

1. For small tasks, a Micro Plan inside the assistant response is acceptable.
2. For large or risky tasks, an ExecPlan file is required and must be created or updated before any further action.
3. If new files, commands, or scope are discovered, the plan must be updated and acknowledged again.
4. Plan Self Check must be completed before any file reads, searches, or commands.

## Plan Types

1. Micro Plan
   Details: For isolated, low risk changes (single file or trivial edits). Must list files to read and commands to run. Does not require a plan file.
2. ExecPlan
   Details: For multi step, cross crate or package, or risky work. Must be stored as a plan file and kept current.

## Org Task Tracking

Org is the authoritative active task-management surface for agents.

1. Active task ledger: `$PRJ_CACHE_HOME/agent/org/agenda.org`.
2. Lane-specific task files: `$PRJ_CACHE_HOME/agent/org/<slug>.org`, allowed
   when the main agenda links or includes them.
3. Template: `$PRJ_ROOT/.agent/org/_task_template.org`.

Each active implementation heading should use native Org syntax:

1. Lifecycle keyword: `TODO`, `NEXT`, `WAITING`, `DONE`, or `CANCELLED`.
2. Planning timestamps: `SCHEDULED`, `DEADLINE`, and `CLOSED` when applicable.
3. A `:PROPERTIES:` drawer with relevant fields such as `BLUEPRINT`,
   `EXECPLAN`, `STABLE_REF`, `PACKAGE`, `SLICE`, `STATUS`, and `EVIDENCE`.
4. Progress cookies such as `[1/3]` and `[33%]` on headings that contain
   direct checklists, direct TODO/DONE child headings, or both.
5. Native checkbox items such as `- [ ]` and `- [X]` for task-local work.
6. `COOKIE_DATA: direct` in the property drawer when a heading mixes direct
   TODO/DONE child headings and direct checkbox progress. This keeps the cookie
   heading-local and does not count checklists inside child headings.
7. No empty property values in templates; use explicit placeholders such as
   `<execplan-path>` or `none`.
8. Short log entries under the heading for implementation checkpoints,
   validation runs, and handoff notes.

Use `.agent/org/README.org` as the workflow reference for active recovery,
completion, archive, and achievement queries.

## When an ExecPlan Is Required

1. The task spans multiple crates, packages, or subsystems.
2. The task is expected to take multiple implementation steps with checkpoints.
3. The task has architectural ambiguity or nontrivial tradeoffs.
4. The task is risky (regression risk, migration risk, or production facing behavior change).

Small, isolated fixes do not require a full ExecPlan.

## Where Plans Live

1. Policy file: `.agent/PLANS.md` (this file).
2. Template file: `.agent/execplans/_template.org`.
3. Active plans: `$PRJ_CACHE_HOME/agent/execplans/<slug>.org`.
4. Archived DONE plans: `$PRJ_CACHE_HOME/agent/execplans/archives/<slug>.org`.
5. Active Org task ledger: `$PRJ_CACHE_HOME/agent/org/agenda.org`.
6. Org task template: `.agent/org/_task_template.org`.

## Required Plan Structure

Each ExecPlan Org file should contain these headings:

1. `#+TITLE: ExecPlan Title`
2. `* TODO ExecPlan Title [0/N] [0%]` with `COOKIE_DATA: direct`
3. `** Purpose / Big Picture`
4. `** Scope and Boundaries`
5. `** Plan Self Check`
6. `** Context and Orientation`
7. `** Plan of Work`
8. `** Concrete Steps`
9. `** Validation and Acceptance`
10. `** Reflection and Quality Audit`
11. `** Final Validation Gate`
12. `** Idempotence and Recovery`
13. `** Interfaces and Dependencies`
14. `** Progress`
15. `** Decision Log`
16. `** Surprises & Discoveries`
17. `** Artifacts and Notes`
18. `** Outcomes & Retrospective`
19. `** Change Log`
20. `** Recovery Commands`

## Scope and Boundaries (Required Detail)

This section must include:

1. Files or dirs to read
2. Commands or tools to run (including tests)
3. Expected outputs
4. Stop conditions

Any activity outside this scope requires a plan update and reacknowledgement.

## Plan Self Check (Required)

This section must include:

1. A progress cookie on the heading, for example `[0/8] [0%]`.
2. Native Org checkbox items for each self-check item.
3. `COOKIE_DATA: direct` when the heading also has direct TODO/DONE children.
4. Scope matches the request and risk level.
5. Files or dirs to read are complete and minimal.
6. Commands or tools to run are complete and safe.
7. Expected outputs are concrete and testable.
8. Stop conditions are clear.
9. Dependencies and constraints are recorded.
10. Validation plan is adequate for risk.
11. Plan type is correct.

Work must not proceed until this self check is complete.

## Authoring Rules

1. Keep the plan self contained so a new contributor can execute it without prior context.
2. Update `** Progress`, `** Decision Log`, `** Change Log`, and the paired
   Org task heading as work advances.
3. Prefer concrete checkpoints over vague statements.
4. Include exact verification commands in `** Validation and Acceptance`.
5. Record rollback or retry behavior in `** Idempotence and Recovery`.

## Orgize Validation

Agent tracking files are native Org documents. When changing files under
`$PRJ_CACHE_HOME/agent/org/`, `$PRJ_CACHE_HOME/agent/blueprints/`, or
`$PRJ_CACHE_HOME/agent/execplans/`, prefer the orgize-backed project entrypoint
for lint and query checks before marking tracking work complete. Install or
refresh the client with `direnv exec . just install-wendao-client`, then run:
`wendao-client orgize lint --format compact <path>`.

Use native Org agenda semantics for schedule/task lookup:
`wendao-client orgize agent-planning --date YYYY-MM-DD <path>`.

Use native Org sparse-tree semantics for task-local search:
`wendao-client orgize sparse-tree --match '+agent' --exclude-done <path>`.

Use the DuckDB-backed Org task list for active recovery:
`wendao-client orgize task-list [--text TEXT] [--tag TAG] <path>`.

Use the DuckDB-backed Org task report for archive and achievement summaries:
`wendao-client orgize task-report [--text TEXT] [--tag TAG] <path>`.

Use the DuckDB-backed Org task archive command for plan-first physical
archival. Omit `--apply` for read-only planning:
`wendao-client orgize task-archive [--apply] [--text TEXT] [--tag TAG] <path>`.

## Resume and Archive Commands

Recover active tasks:

`wendao-client orgize task-list $PRJ_CACHE_HOME/agent/org`.

Recover scheduled tasks:

`wendao-client orgize agent-planning --date YYYY-MM-DD $PRJ_CACHE_HOME/agent/org`.

Recover one lane or package:

`wendao-client orgize task-list --text '<lane-or-package>' $PRJ_CACHE_HOME/agent/org`.

Review completed achievements:

`wendao-client orgize task-list --tag achievement --include-done $PRJ_CACHE_HOME/agent/org`.

Review archive candidates and repeating task counts:

`wendao-client orgize task-report $PRJ_CACHE_HOME/agent/org`.

Plan physical archival:

`wendao-client orgize task-archive $PRJ_CACHE_HOME/agent/org`.

Use sparse-tree when full source context is needed:

`wendao-client orgize sparse-tree --match '+agent+achievement' $PRJ_CACHE_HOME/agent/org`.

## Reflection and Quality Audit (Required)

This section must include:

1. Code audit: correctness, reliability, security, performance, and maintenance risk.
2. Plan audit: scope adherence, deviations, and any unexecuted steps.
3. Verification audit: what ran, what did not run, and why that is acceptable.

## Final Validation Gate

This section must be the last checkpoint before marking work DONE.

1. Confirm `** Validation and Acceptance` is complete.
2. Confirm `** Reflection and Quality Audit` is recorded.
3. State a final go or no go decision with rationale.

## Quick Start

1. Copy `.agent/execplans/_template.org` to `$PRJ_CACHE_HOME/agent/execplans/<slug>.org`.
2. Create or update the paired Org heading in `$PRJ_CACHE_HOME/agent/org/agenda.org` using `.agent/org/_task_template.org`.
3. Fill `Purpose`, `Scope and Boundaries`, `Context`, and `Plan of Work` before coding.
4. Record `NEXT_ACTION` and `RESUME_QUERY` before leaving a turn.
5. Keep the ExecPlan and Org task heading current until the initiative is complete.
6. At completion, mark the task `DONE`, add `CLOSED`, record evidence, and archive the ExecPlan.
