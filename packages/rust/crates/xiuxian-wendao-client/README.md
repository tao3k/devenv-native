# xiuxian-wendao-client

`xiuxian-wendao-client` is the lightweight Wendao user CLI surface.

Current scope:

1. local-only commands
2. no gateway or server bootstrap
3. reusable clap command types that can be embedded into `xiuxian-wendao`

## Current Commands

The first landed command is:

```text
wendao lint markdown [PATH]...
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
5. classifies diagnostics as `official_syntax` or `repo_authoring_policy`
6. reports invalid YAML frontmatter
7. reports unclosed frontmatter blocks
8. reports unclosed fenced code blocks
9. treats mixed `[[target]](label)` link syntax as an official-syntax failure
10. treats bare `[[target]]`, redundant labels such as `[[target|target]]`, and target-like reversed alias shapes such as `[[label|path/to/doc.md]]` as repo authoring policy findings rather than official Obsidian syntax failures
11. resolves reachable link targets to document titles and heading fragments so diagnostics can suggest a concrete rewrite for LLM repair flows
12. emits plain-text diagnostics for human and LLM review
13. reports non-UTF-8 Markdown files
14. keeps official Obsidian embeds such as `![[note]]`, `![[note#Heading]]`,
    and `![[note#^block-id]]` parser-compatible and outside the ordinary
    authoring-policy lint lane
15. adds a directory-level authoring policy so one folder does not mix
    explicit Obsidian note links and standard Markdown note links
16. when a directory style mismatch is found, emits a precise rewrite for the
    offending link instead of a style-only hint

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

The standalone binary is also named `wendao`. When `xiuxian-wendao` depends on
this crate, the same client subcommands can be flattened into the main Wendao
CLI without duplicating execution logic.
