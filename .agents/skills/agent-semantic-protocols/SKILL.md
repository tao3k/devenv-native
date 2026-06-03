---
name: agent-semantic-protocols
description: Use when working with the language provider binaries maintained by agent-semantic-protocols, including rs-harness, ts-harness, py-harness, semantic-agent-hook installs, compact semantic search flow, and non-JSON agent command guidance.
---

# Agent Semantic Protocols

## Rules

- Choose the bin from the project language: Rust uses `rs-harness`, TypeScript uses `ts-harness`, Python uses `py-harness`.
- Julia is intentionally skipped for bin parity and cross-language search alignment for now. Do not require, install, document, or validate a standalone `julia-project-harness` binary.
- If Julia evidence is explicitly requested, use the workspace-managed command `julia --project=languages/JuliaLangProjectHarness.jl languages/JuliaLangProjectHarness.jl/bin/julia-project-harness.jl`; otherwise skip Julia in provider parity audits.
- Start with `<bin> agent guide .` when unsure; it prints the provider-owned command menu.
- Do not add `--json` during agent exploration. `--json` is only for schema tests, validators, receipts, or IDE integrations.
- `semantic-agent-hook install --client codex .` installs hooks, provider activation, and this skill at `.agents/skills/agent-semantic-protocols/SKILL.md`.

## Command Shapes

- Map the project: `<bin> search prime --view seeds .`
- Resolve an owner: `<bin> search owner <owner-path> --view seeds .`
- Search local fuzzy text with tests: `<bin> search fzf <term> owner tests --view seeds .`
- Search external API/deps: `<bin> search deps <dep[/subpath][@version][::api]> .`
- Query parser items with compact code: `<bin> search owner <path> items --query '<symbol-or-a|b|c>' .`
- Discover owner-local item names before code: `<bin> query <path> --term <candidate> --names-only .`
- Follow a hook exact direct-read route: `<bin> query --from-hook direct-source-read --selector <path[:line-range]> [--code] .`
- Follow a hook wildcard direct-read route: `<bin> search query --from-hook direct-source-read --selector <glob-or-path> --term <term> --surface owner,tests --view seeds .`
- Pipe candidate lines: `rg -n '<term>' src tests | <bin> search ingest --view seeds .`
- Check changed work: `<bin> check --changed .`

Hook rule of thumb: source-suffix reads and content dumps are denied; exact
source paths should follow the provider `query --from-hook direct-source-read
--selector <path[:line-range]> [--code] .` route; raw
`rg`/`grep`/`fd`/`find` with a concrete source term should follow the hook
`search query` route; source file listings without terms should pipe candidates
to `search ingest`; non-source docs/README/markdown searches should be allowed.
Exact direct-read may return either `read-owner` source windows or a `read-plan`
with `code=false`. When it returns `read-plan`, follow the provider-selected
`|symbol read=...` or search repair action instead of forcing a raw source dump.

When the provider guide advertises handle-aware search, use it before code for
stable non-code facts such as policy rule ids, schema fixtures, test cases,
config keys, command surfaces, dependency APIs, or capabilities:
`<bin> search policy <rule-id-or-alias> owner tests --view seeds .` or
`<bin> search owner <owner-path> handles --query <term> .`

Owner item query output includes a `|query` line with `status=hit|miss` and
`match=exact|fallback-contains|none`. Treat `status=miss` as a wrong or stale
symbol query to revise, not as permission to keep raw-searching the file. If a
miss line includes `candidates=...`, follow the parser-owned candidate instead
of guessing another symbol. Use `--names-only` for broad owner-local prefixes
such as `parse_` so the provider returns item names and read locators without
dumping code windows.

When `searchSynthesis.windowSet` appears, treat it as the provider-selected
bounded read plan. Read those exact owner/test targets with
the provider `query --from-hook direct-source-read --selector <path[:line-range]> [--code] .` route
only when source owner context is needed; do not restart broad discovery from
the same terms.

Rust owner and ingest accept extra scopes:

```sh
rs-harness search owner src/lib.rs items --view seeds .
rg -n 'HookDecision' src tests | rs-harness search ingest items tests --view seeds .
```

Julia is skipped in bin parity audits. Only use the workspace-managed provider
command when a Julia-specific task explicitly asks for Julia evidence:

```sh
julia --project=languages/JuliaLangProjectHarness.jl languages/JuliaLangProjectHarness.jl/bin/julia-project-harness.jl search owner src/cli.jl --view seeds languages/JuliaLangProjectHarness.jl
printf 'src/cli.jl:1\n' | julia --project=languages/JuliaLangProjectHarness.jl languages/JuliaLangProjectHarness.jl/bin/julia-project-harness.jl search ingest owner tests --view seeds languages/JuliaLangProjectHarness.jl
```

## Flow Examples

1. Implement a TypeScript feature around a known symbol:

```sh
ts-harness agent guide .
ts-harness search prime --view seeds .
ts-harness search fzf runCodexAgentHook owner tests --view seeds .
ts-harness search owner src/cli/agent-hooks.ts --view seeds .
ts-harness check --changed .
```

Use the `owner` output to choose the edit file. Use the `tests` seeds before editing.

2. Find Rust API usage before changing behavior:

```sh
rs-harness agent guide .
rs-harness search deps tokio::spawn public-api --view seeds .
rs-harness search fzf tokio::spawn tests --view seeds .
rs-harness search owner src/runtime.rs items --query RuntimeConfig .
```

Use `deps` for external API facts. Use `owner` only after a real owner path appears.
Use provider-native `search owner <path> items --query <symbol>` for compact code extraction.
Do not use raw `cat`, `sed`, `rtk read`, or editor reads for source files.

3. Understand a Python implementation path:

```sh
py-harness agent guide .
py-harness search prime --view seeds .
rg -n 'Session' src tests | py-harness search ingest --view seeds .
py-harness search owner src/client.py --view seeds .
py-harness search owner src/client.py items --query 'Session|request' .
py-harness check --changed .
```

Use `rg` or `fd` to collect candidates, then let `py-harness` rank owners/tests.

Julia is out of scope for bin-parity flows. Do not use a bare
`julia-project-harness` command in docs, tests, hook activations, or provider
parity audits. Julia remains workspace-managed until a separate Julia-specific
lane is reopened.

## Combination Query Examples

1. Same TypeScript question, multiple names:

```sh
ts-harness search fzf --query-set runCodexAgentHook --query-set permissionDecision owner tests --view seeds .
```

Use this when both terms describe the same hook decision path.
For a hook-blocked wildcard read such as `Read *.ts`, use the hook query form
when the selector is broad and the agent has concrete terms:

```sh
<bin> search query --from-hook direct-source-read --selector '**/*.{ts,tsx,js}' --term parseSearchArgs --term querySets --surface owner,tests --view seeds .
```

This emits normal search seeds and synthesis; it is not a raw source-read
fallback.

2. Same Rust concept, type plus field:

```sh
rs-harness search fzf --query-set HookDecision --query-set permissionDecision owner tests --view seeds .
```

Use this when one answer should cover both aliases. Follow with `rs-harness search owner <path> items --query '<symbol|otherSymbol>' .` when a concrete owner is selected.

3. Same Python API, import name plus method name:

```sh
py-harness search fzf --query-set requests.Session --query-set Session.request owner tests --view seeds .
```

Use this when API naming varies across imports/callsites.

Do not use query-set for independent axes. Run these separately and synthesize after reading compact outputs:

```sh
ts-harness search deps playwright::APIRequestContext .
ts-harness search owner src/http/client.ts --view seeds .
ts-harness search tests src/http/client.ts --view seeds .
```
