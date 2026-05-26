# xiuxian-wendao-client

`xiuxian-wendao-client` is the Wendao user CLI surface for local document
tooling and read-model operations.

Current scope:

1. local-only commands
2. no gateway or server bootstrap
3. reusable clap command types that can be embedded into `xiuxian-wendao`

## Current Commands

The currently landed commands are:

```text
wendao-client lint markdown [PATH]...
wendao-client orgize fmt [--check] [PATH]...
wendao-client orgize lint [--format compact|text|json] [--json] [PATH]...
wendao-client orgize agent-planning --date YYYY-MM-DD [--end YYYY-MM-DD] [--include-done] [--include-archived] [--include-comments] [--match EXPR] [PATH]...
wendao-client orgize read-model [PATH]...
wendao-client orgize task-list [--text TEXT] [--tag TAG]... [--include-done] [--include-archived] [--limit N] [PATH]...
wendao-client orgize orgid-show --id ORGID [--full] [PATH]...
wendao-client orgize task-report [--text TEXT] [--tag TAG]... [--include-archived] [--limit N] [PATH]...
wendao-client orgize task-archive [--apply] [--text TEXT] [--tag TAG]... [--limit N] [PATH]...
wendao-client orgize sparse-tree [--text TEXT] [--match EXPR] [--exclude-done] [--exclude-archived] [--include-comments] [--explain-skips] [PATH]...
wendao-client orgize sdd status [PATH]...
wendao-client get toc [TARGET] [--ignore DIR]...
wendao-client get page-index [TARGET] [--ignore DIR]...
```

The semantic SQL command surface remains feature-gated and is not part of the
default installed client. When enabled, it uses the `xiuxian-wendao-sql`
DuckDB/Arrow local relation API.

```text
wendao-client lint semantic [--read-model-summary] [--semantic-sql-guard] [--projection-refresh-plan] [--require-fresh-projections] [--refresh-projections] [--lifecycle-plan] [--apply-lifecycle-plan]
wendao-client lint semantic [--read-model-summary] [--semantic-sql-guard] [--projection-refresh-plan] [--require-fresh-projections] [--refresh-projections] [--lifecycle-plan] [--apply-lifecycle-plan] [PATH]...
wendao-client semantic describe-read-model [PATH]
wendao-client semantic snapshot-read-model [PATH]
wendao-client semantic check-read-model-snapshot --expect SNAPSHOT_REVISION [PATH]
wendao-client semantic plan-read-model-materialization [--expect-snapshot SNAPSHOT_REVISION] [PATH]
wendao-client semantic preflight-read-model-materialization [--expect-snapshot SNAPSHOT_REVISION] [PATH]
wendao-client semantic query-read-model --query SQL [PATH]
wendao-client semantic refresh-projections [--interval-secs SECONDS] [--max-runs RUNS] [--require-clean-worktree] [PATH]...
```

Behavior:

1.  walks Markdown files under the provided paths, or when no paths are given,
    loads local writable `link_graph.projects.*.root` entries from
    `wendao.toml` before falling back to the configured root
2.  treats `link_graph.projects.<id>.read_only = true` as the canonical way to
    exclude a configured project root from default lint discovery
3.  honors `link_graph.projects.<id>.read_only = false` even when the project
    also declares `url`, so writable mirrors can still be linted by default
4.  keeps `url`-based managed-remote detection only as a backward-compatible
    readonly inference when `read_only` is omitted
5.  skips transient/generated directories such as `.cache`, `.data`, `.run`,
    `.config`, `.bin`, `node_modules`, and `target` during default markdown
    discovery
6.  classifies diagnostics as `official_syntax` or `repo_authoring_policy`
7.  requires document-level YAML frontmatter and validates the primary
    frontmatter identity field for the current document surface:
    - ordinary Markdown documents require a non-empty `title`
    - `SKILL.md` files or `kind: SKILL.md` documents must satisfy the
      parser-owned SKILL.md frontmatter contract: top-level `type: skill`,
      `name`, and `description`; top-level `metadata`; non-empty
      `metadata.author`, `metadata.version`, `metadata.source`; and a
      non-empty `metadata.routing_keywords` array; optional
      `metadata.intents` must still be a non-empty string array when present
8.  reports invalid YAML frontmatter
9.  reports unclosed frontmatter blocks
10. reports unclosed fenced code blocks
11. fails when a local Markdown link, wikilink, or attachment target does not
    resolve to an existing in-scope file
12. fails when a reachable local link or attachment resolves into a
    transient/generated repository directory
13. fails when a local file target resolves but the addressed heading fragment
    or `#^block-id` fragment is missing
14. rejects local note and attachment targets that escape the active lint root
    via `..` traversal, even when the escaped file exists
15. treats mixed `[[target]](label)` link syntax as an official-syntax failure
16. treats bare `[[target]]`, redundant labels such as `[[target|target]]`, and target-like reversed alias shapes such as `[[label|path/to/doc.md]]` as repo authoring policy findings rather than official Obsidian syntax failures
17. resolves reachable link targets to document titles and heading fragments so diagnostics can suggest a concrete rewrite for LLM repair flows
18. emits compact ariadne-style source diagnostics by default for human and
    LLM review, with JSON formats remaining available through `--output json`
    and `--output pretty`
19. reports non-UTF-8 Markdown files
20. keeps official Obsidian embeds such as `![[note]]`, `![[note#Heading]]`,
    and `![[note#^block-id]]` parser-compatible and outside the ordinary
    authoring-policy lint lane
21. adds a directory-level authoring policy so one folder does not mix
    explicit Obsidian note links and standard Markdown note links
22. when a directory style mismatch is found, emits a precise rewrite for the
    offending link instead of a style-only hint
23. in `semantic-sql` builds, validates repo-native semantic SSOT roots with `lint semantic`, defaulting
    to `semantic/` when no path is supplied. Explicit `[PATH]...` arguments are
    only needed for custom semantic roots. The command fails on invalid object
    frontmatter, duplicate IDs,
    unresolved relation targets, unresolved projection source objects, empty
    owner/provenance/verification fields, and invalid active confidence
    sources; fresh projection artifacts must also declare the current source
    revision, while stale projections must be explicitly marked stale; optional
    `semantic/change-intents/` artifacts must resolve touched objects,
    relation endpoints, landed status transitions, affected invariants,
    projection refresh targets, candidate suggestion IDs, and explicit
    promotion/demotion target outcomes
24. refreshes semantic projection `source_revision` and `staleness` metadata
    only when `lint semantic --refresh-projections` is passed; this is an
    explicit derived-metadata writeback and does not regenerate projection
    bodies or make projections authoritative
25. renders a read-only lifecycle writeback preview when
    `lint semantic --lifecycle-plan` is passed, listing validated promotion,
    demotion, and other status-transition outcomes without mutating semantic
    object files
26. applies pending lifecycle status transitions only when
    `lint semantic --apply-lifecycle-plan` is passed. The object current
    status must match the declared transition `from` status; promotion also
    rewrites `confidence.source` to `human_signed` and removes the promoted
    object from change-intent `candidate_suggestions`
27. requires projections named by active change intents to be fresh when
    `lint semantic --require-fresh-projections` is passed. This is a
    closure-level policy gate; ordinary semantic lint still accepts explicitly
    stale advisory projections
28. renders a read-only projection metadata refresh plan when
    `lint semantic --projection-refresh-plan` is passed. This is the
    parser-owned queue contract for future background refresh workers; the
    command does not mutate projection artifacts unless `--refresh-projections`
    is also passed explicitly
29. runs an explicit semantic projection metadata refresh worker with
    `semantic refresh-projections`. The default remains one pass. Passing
    `--interval-secs` makes the same worker run as a recurring local runner;
    `--max-runs` bounds repeated runs for verification or supervised jobs.
    `--require-clean-worktree` makes supervised starts fail before any
    projection writeback when the root git worktree already has pending
    changes. Each pass uses the same semantic lint engine, applies projection
    metadata refresh through the existing writeback path, renders the
    post-refresh plan, and requires projection freshness before returning
    success. The project `process-compose` surface packages this runner as
    `wendao-semantic-refresh`, with managed pid/log state and
    `WENDAO_SEMANTIC_REFRESH_INTERVAL_SECS` /
    `WENDAO_SEMANTIC_REFRESH_MAX_RUNS` operator controls
30. renders an advisory semantic read-model summary when
    `lint semantic --read-model-summary` is passed, showing row counts for
    `semantic_objects`, `semantic_relations`, and
    `semantic_projection_state` while leaving repo-native semantic artifacts
    authoritative
31. describes the advisory semantic read-model catalog with
    `semantic describe-read-model`, defaulting to the active `semantic/` root
    and rendering stable table, column, nullability, and row-count metadata
    for `semantic_objects`, `semantic_relations`, and
    `semantic_projection_state` without registering a query or mutating
    semantic artifacts
32. renders deterministic advisory semantic read-model snapshot revisions with
    `semantic snapshot-read-model`, defaulting to the active `semantic/` root.
    The snapshot covers table schemas and projected rows for
    `semantic_objects`, `semantic_relations`, and
    `semantic_projection_state`; it is evidence for future snapshot-swap
    read-model work and does not make hashes or SQL authoritative
33. checks the current advisory semantic read-model snapshot revision with
    `semantic check-read-model-snapshot --expect SNAPSHOT_REVISION`,
    defaulting to the active `semantic/` root. The check exits zero on exact
    aggregate revision match and non-zero on mismatch while rendering the
    current table revisions for operator review. This is a read-only evidence
    guard for future snapshot-swap work and does not make hashes or SQL
    authoritative
34. renders a read-only future materialization plan with
    `semantic plan-read-model-materialization`, defaulting to the active
    `semantic/` root. The plan targets a future DuckDB snapshot-swap read
    model, lists the current aggregate and table revisions, and can be gated
    with `--expect-snapshot SNAPSHOT_REVISION`. A mismatched expected snapshot
    returns a blocked plan and a non-zero exit status. The command does not
    register DuckDB tables, write derived state, or make DuckDB authoritative
35. executes a read-only materialization preflight with
    `semantic preflight-read-model-materialization`, defaulting to the active
    `semantic/` root. The preflight reuses the snapshot gate, registers the
    three advisory semantic read-model tables into the request-scoped local
    relation engine, runs a smoke query, and reports runtime registration
    evidence. It still writes no derived state and does not make DuckDB
    authoritative
36. executes read-only SQL over advisory semantic read-model tables with
    `semantic query-read-model --query SQL`, defaulting to the active
    `semantic/` root and rendering text, JSON, or pretty JSON through the
    global output option. The registered tables are `semantic_objects`,
    `semantic_relations`, and `semantic_projection_state`; query results are
    evidence only and do not mutate semantic artifacts. The SQL crate admits
    exactly one DuckDB-dialect read-only query statement and rejects blank,
    multi-statement, or mutation SQL before table registration. The local
    relation engine is DuckDB.
37. exposes upstream Orgize tooling through `orgize fmt`, `orgize lint`,
    `orgize agent-planning`, `orgize read-model`, `orgize task-list`,
    `orgize orgid-show`, `orgize task-probe`, `orgize task-report`,
    `orgize task-archive`, `orgize sparse-tree`, and `orgize sdd status`.
    Formatting and linting use parser-owned Orgize adapters from
    `xiuxian-wendao-parsers`; planning and sparse-tree commands render compact
    cards derived from native Org agenda and sparse-tree semantics. The `sdd
status` command renders Org-native SDD
    system/capability/view/decision/audit architecture status and parent edges
    from upstream Orgize SDD parsing. The `read-model` command materializes
    agent-tagged Org tasks into DuckDB by default and also writes the derived
    `agent_org_memory_objects` table. Memory object rows keep `orgid` as the
    source section identity and use `source_kind`, `source_key`, and
    `object_index` for row disambiguation; no separate `memory_id` or
    `task_orgid` identity is introduced. The command does not expose a
    `--duckdb` runtime selector. The `task-list` command refreshes the same
    DuckDB read model and renders active task rows for agent recovery, with
    optional text/tag filtering and explicit DONE/archive inclusion flags.
    Text filtering also considers the derived memory object kind, facet,
    source key, and value fields through the source `orgid`. The `task-probe`
    command is the compact
    remembered-task recovery view; it reranks candidates with
    `xiuxian-memory-engine` plus structured Org facet fusion so title text,
    properties, SDD references, checklist state, and next actions vote together
    instead of letting one noisy title phrase dominate. The `task-list` and
    `orgid-show` JSON outputs include inferred `memoryObjects` derived from
    typed Org properties and completed Reflection Questions rows; the memory
    engine owns the object classification and the client only projects Org
    evidence. This is the serverless memory path: copied or normalized
    reference-memory samples become project-owned Org properties, then
    `wendao-client orgize read-model` materializes them into DuckDB/Arrow-ready
    rows for compact local recall. Rows marked with memory lifecycle properties
    such as `MEMORY_STATUS: superseded`, `MEMORY_STATUS: stale`,
    `MEMORY_STATUS: rejected`, or `MEMORY_STATUS: blocked` stay visible as Org
    tasks but are excluded from derived memory objects and serverless recall
    packets. The client does not read external Codex memory files at runtime.
    The `task-report`
    command summarizes the same snapshot for active rows, completed
    achievements, archive candidates, repeating rows, and tag counts. The
    `task-archive` command renders an archive plan by default and only mutates
    Org source when `--apply` is passed; applied tasks are moved to
    `archives/<source-task-file>.org` by default, receive the native `ARCHIVE`
    tag, and are appended as raw Org subtrees without synthesized archive-file
    metadata headers. Deprecated yearly bucket targets such as `2026.org` are
    ignored in favor of the source-task-file target. Native Org repeater
    cookies on `SCHEDULED` or
    `DEADLINE` timestamps are preserved and rendered as `repeat:` metadata so
    recurring profile, benchmark, or audit tasks remain visible without a
    custom schedule DSL. When no source path is
    supplied, read-model commands read from `$PRJ_CACHE_HOME/agent/org`; the
    default database is
    `$PRJ_CACHE_HOME/agent/readmodels/org_agent_tasks.duckdb`. Optional
    path/runtime overrides belong in the `wendao.toml` `[agent.org_read_model]`
    table through `database_path`, `temp_directory`, and `threads`. The refresh
    path uses DuckDB's native appender for materialization. The performance
    profile is covered by
    `cargo bench -p xiuxian-wendao-client --features performance --bench wendao_client_orgize`;
    the current 1,024-row refresh plus cached-query profile is roughly
    42-57 ms on the local benchmark fixture. The cached active recovery query
    for a 20-row window is roughly 6.1-6.8 ms.

Diagnostic rendering is split deliberately:

1. parser and client Rust code still own semantic detection, target
   resolution, and rewrite-strategy logic
2. the client implementation is split under `src/lint/diagnostic/` so context
   building, diagnostic facts, link parsing, and text/render helpers stay
   modular
3. `resources/contracts/manifests/wendao.markdown_lint.diagnostics.toml`
   and its checked-in
   `resources/contracts/snapshots/wendao.markdown_lint.diagnostics/{contract.toml,schema.json}`
   pair now own both the CLI invocation contract for
   `wendao lint markdown` and the diagnostic rendering contract for `problem`,
   `detail`, `found`, `expected`, and `tip`, including `invalid_utf8`
4. runtime loading consumes the normalized snapshot `contract.toml`, while
   tests generate `contract.toml` and `schema.json` from the manifest to catch
   contract drift
5. the client-side contract implementation is split under
   `src/lint/contract/` so checked-in assets, normalized snapshot
   types, manifest validation, and schema generation do not live in one flat
   file
6. the TOML layer does not define lint semantics; it only selects stable
   rendering strategies over Rust-collected facts
7. directory-level note-link-style policy stays in `src/lint/policy/`, so
   parser syntax ownership and repository authoring policy remain separate
8. the default text renderer uses ariadne compact ASCII diagnostics over the
   original Markdown source; diagnostic metadata is carried inside ariadne
   notes and helps (`kind`, `problem`, `target`, `expected`, `detail`, `tip`)
   so the compact diagnostic remains the primary LLM repair surface
9. focused text snapshots under `tests/snapshots/` lock the compact diagnostic
   layout for source-backed diagnostics, one no-source fallback, and one
   `wendao-episteme`-style framework repair line without snapshotting every
   lint rule or replacing the full `wendao-episteme/tests` scenario suite

The `get` commands stay local and parser-owned by design:

1. `TARGET` accepts one Markdown file or one directory
2. the client materializes TOC and page-index payloads directly from a
   lightweight parser-owned outline path instead of the heavier full TOC
   aggregation surface
3. default human-facing output is compact Markdown without synthetic mode
   headings such as `# TOC` or `# Page Index`
4. `toc` compact Markdown intentionally stays a flat source-order outline that
   preserves document heading levels through native Markdown `#` markers and
   renders each section as `# Heading -> [Lx a-b]`, while the leading `path:`
   line uses the absolute local file path
5. `page-index` compact Markdown intentionally stays a structure-first
   page-index view that reuses the same heading-plus-range syntax while adding
   parser-preserved `links:` and `embeds:` lines plus document-level
   node/link/embed counts, and the leading `path:` line uses the absolute
   local file path
6. `--output json` and `--output pretty` preserve the structured payloads
7. recursive directory traversal merges:
   - built-in local runtime-dir ignores such as `.cache`, `.data`, `.run`,
     `.config`, `.bin`, `target`, and `node_modules`
   - `link_graph.exclude_dirs` from the active `wendao.toml` config
   - repeatable `--ignore DIR` command-line additions
8. the same command tree can be flattened into the main `wendao` CLI without
   reimplementing execution logic in `xiuxian-wendao`

The standalone binary is named `wendao-client`, while the reusable command
tree remains separate from the main `wendao` CLI so execution logic does not
have to be reimplemented there.

For repeated local use, install the binary once instead of invoking
`cargo run` for each command:

```text
direnv exec . just install-wendao-client
wendao-client orgize lint --format compact .agent/org/agenda.org
wendao-client orgize task-list --text '<lane-or-package>'
```

## Project Policy Gate

`xiuxian-wendao-client` uses `rust-lang-project-harness` for source and unit
project-policy checks without disabled rules. The crate root remains a facade,
`src/get/run.rs` keeps output rendering in an owned child module, and markdown
lint discovery keeps `mod.rs` as an interface-only owner.
