# xiuxian-wendao-client

`xiuxian-wendao-client` is the lightweight Wendao user CLI surface.

Current scope:

1. local-only commands
2. no gateway or server bootstrap
3. reusable clap command types that can be embedded into `xiuxian-wendao`

## Current Commands

The currently landed commands are:

```text
wendao-client lint markdown [PATH]...
wendao-client lint semantic [--semantic-sql-guard] [--refresh-projections] [--lifecycle-plan]
wendao-client lint semantic [--semantic-sql-guard] [--refresh-projections] [--lifecycle-plan] [PATH]...
wendao-client get toc [TARGET] [--ignore DIR]...
wendao-client get page-index [TARGET] [--ignore DIR]...
```

Behavior:

1. walks Markdown files under the provided paths, or when no paths are given,
   loads local writable `link_graph.projects.*.root` entries from
   `wendao.toml` before falling back to the configured root
2. treats `link_graph.projects.<id>.read_only = true` as the canonical way to
   exclude a configured project root from default lint discovery
3. honors `link_graph.projects.<id>.read_only = false` even when the project
   also declares `url`, so writable mirrors can still be linted by default
4. keeps `url`-based managed-remote detection only as a backward-compatible
   readonly inference when `read_only` is omitted
5. skips transient/generated directories such as `.cache`, `.data`, `.run`,
   `.config`, `.bin`, `node_modules`, and `target` during default markdown
   discovery
6. classifies diagnostics as `official_syntax` or `repo_authoring_policy`
7. requires document-level YAML frontmatter and validates the primary
   frontmatter identity field for the current document surface:
   - ordinary Markdown documents require a non-empty `title`
   - `SKILL.md` files or `kind: SKILL.md` documents must satisfy the
     parser-owned SKILL.md frontmatter contract: top-level `type: skill`,
     `name`, and `description`; top-level `metadata`; non-empty
     `metadata.author`, `metadata.version`, `metadata.source`; and a
     non-empty `metadata.routing_keywords` array; optional
     `metadata.intents` must still be a non-empty string array when present
8. reports invalid YAML frontmatter
9. reports unclosed frontmatter blocks
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
23. validates repo-native semantic SSOT roots with `lint semantic`, defaulting
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
tree remains small enough to embed into the main `wendao` CLI without pulling
`xiuxian-wendao` back into the client crate.

## Project Policy Gate

`xiuxian-wendao-client` uses `rust-lang-project-harness` for source and unit
project-policy checks without disabled rules. The crate root remains a facade,
`src/get/run.rs` keeps output rendering in an owned child module, and markdown
lint discovery keeps `mod.rs` as an interface-only owner.
