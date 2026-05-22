# Execution Plan Policy

This repository uses ExecPlans for work that is large, uncertain, or cross
cutting. Active agent task state, scheduling, deadlines, and implementation
continuity are tracked in Org files under `$PRJ_CACHE_HOME/agent/org/`.
This repository uses Org as the agent tracking surface.

## Org-native SDD Governance (Preferred)

New complex architecture, migration, or multi-slice work SHOULD start from an
Org-native SDD architecture description. The SDD graph is:

1. System SDD: durable system boundary, quality attributes, and design concerns.
2. Capability SDD: behavior contract for one capability or bounded context.
3. View SDD: architecture viewpoint such as runtime, data, deployment, or integration.
4. Decision SDD: design decision, tradeoff, and rationale.
5. Audit SDD: architecture fitness criteria and drift checks.

The default SDD template lives at `.agent/sdd/_architecture_template.org`.
Active SDD work belongs under `$PRJ_CACHE_HOME/agent/sdd/`, with superseded
design descriptions archived under `$PRJ_CACHE_HOME/agent/sdd/archives/YYYY.org`
or another reviewed archive file.

SDD nodes use native Org properties:

1. `ID` is the stable UUID/ULID machine identity.
2. `SDD_KIND` is `system`, `capability`, `view`, `decision`, or `audit`.
3. `SDD_PARENT` is a semantic Org `id:` link to the parent SDD.
4. `SDD_CAPABILITY`, `SDD_VIEWPOINT`, `SDD_CONCERN`, `SDD_QUALITY`,
   `SDD_RATIONALE`, `SDD_SLUG`, and tags provide search dimensions.
5. SDD headings do not carry `TODO`, progress cookies, or task checklists;
   implementation state belongs in Org task or ExecPlan headings.

Validate SDD files with:
`wendao-client orgize lint --format compact $PRJ_CACHE_HOME/agent/sdd`.

Recover SDD status with:
`wendao-client orgize sdd status $PRJ_CACHE_HOME/agent/sdd`.

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
3. A `:PROPERTIES:` drawer with relevant fields such as `SDD`,
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
2. SDD template file: `.agent/sdd/_architecture_template.org`.
3. Active SDD files: `$PRJ_CACHE_HOME/agent/sdd/<slug>.org`.
4. Template file: `.agent/execplans/_template.org`.
5. Active plans: `$PRJ_CACHE_HOME/agent/execplans/<slug>.org`.
6. Archived DONE plans: `$PRJ_CACHE_HOME/agent/execplans/archives/<slug>.org`.
7. Active Org task ledger: `$PRJ_CACHE_HOME/agent/org/agenda.org`.
8. Org task template: `.agent/org/_task_template.org`.

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
`$PRJ_CACHE_HOME/agent/org/`, `$PRJ_CACHE_HOME/agent/sdd/`,
or `$PRJ_CACHE_HOME/agent/execplans/`,
prefer the orgize-backed project entrypoint for lint and query checks before
marking tracking work complete. Install or refresh the client with
`direnv exec . just install-wendao-client`, then run:
`wendao-client orgize lint --format compact <path>`.

Use native Org agenda semantics for schedule/task lookup:
`wendao-client orgize agent-planning --date YYYY-MM-DD <path>`.

Use native Org sparse-tree semantics when the full source subtree context is
needed:
`wendao-client orgize sparse-tree --match '+agent' --exclude-done <path>`.

Use Org-native SDD status for system/capability/view/decision/audit recovery:
`wendao-client orgize sdd status <path>`.

Use the DuckDB-backed Org task list for active recovery. Add `--cached` for
fast Codex turn-start recovery from an existing snapshot; omit it when the
query must refresh from Org source files first:
`wendao-client orgize task-list [--cached] [--text TEXT] [--tag TAG] <path>`.
Use `--output json` or `--output pretty` when automation needs limited task
rows as a machine-readable recovery contract.
Use named task-list views for common recovery and archive control surfaces:
`wendao-client orgize task-list --cached --view closure-needed <path>`,
`wendao-client orgize task-list --cached --view archive-candidate <path>`,
`wendao-client orgize task-list --cached --view achievement <path>`, and
`wendao-client orgize task-list --cached --view repeating <path>`.

Use the DuckDB-backed Org task report for archive and achievement summaries:
`wendao-client orgize task-report [--cached] [--summary-only] [--text TEXT] [--tag TAG] <path>`.
Use `--output json` or `--output pretty` when automation needs the report
summary as a machine-readable contract.
The same named views apply when a focused summary is more useful than the full
report:
`wendao-client orgize task-report --cached --view archive-candidate <path>`.

Use the DuckDB-backed Org task archive command for plan-first physical
archival. Omit `--apply` for read-only planning:
`wendao-client orgize task-archive [--apply] [--expect-selected COUNT] [--text TEXT] [--tag TAG] <path>`.
Use `--expect-selected` after reviewing a plan so apply fails before writes if
the selected row count changes.
Use `--output json` or `--output pretty` in plan mode when automation needs the
selected rows and archive target counts as a machine-readable review contract.
When `--apply` is used, the command prints updated source/target counts and
refreshes the DuckDB read model before returning. In JSON or pretty output
mode, apply returns the same write receipt as a machine-readable object.

## Resume and Archive Commands

Recover active tasks:

`wendao-client orgize task-list --cached $PRJ_CACHE_HOME/agent/org`.

Refresh active tasks after editing Org files:

`wendao-client orgize task-list $PRJ_CACHE_HOME/agent/org`.

Recover scheduled tasks:

`wendao-client orgize agent-planning --date YYYY-MM-DD $PRJ_CACHE_HOME/agent/org`.

Recover one lane or package:

`wendao-client orgize task-list --cached --text '<lane-or-package>' $PRJ_CACHE_HOME/agent/org`.

Review completed achievements:

`wendao-client orgize task-list --cached --view achievement $PRJ_CACHE_HOME/agent/org`.

Review archive candidates and repeating task counts:

`wendao-client orgize task-report --cached --summary-only $PRJ_CACHE_HOME/agent/org`.

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

1. Copy `.agent/sdd/_architecture_template.org` to `$PRJ_CACHE_HOME/agent/sdd/<slug>.org` for new complex work.
2. Replace template IDs with real UUID/ULID values and set `SDD_PARENT` links.
3. Create or update the paired Org heading in `$PRJ_CACHE_HOME/agent/org/agenda.org` using `.agent/org/_task_template.org`.
4. Use `.agent/execplans/_template.org` only when the slice still needs detailed execution logging beyond the SDD architecture description.
5. Record `NEXT_ACTION` and `RESUME_QUERY` before leaving a turn.
6. Keep the SDD and Org task heading current until the initiative is complete.
7. At completion, mark the task `DONE`, add `CLOSED`, record evidence, and archive the ExecPlan if one exists. Archive the SDD only when the design is superseded or retired.
