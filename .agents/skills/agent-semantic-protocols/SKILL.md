---
name: agent-semantic-protocols
description: Use when working with the language provider binaries maintained by agent-semantic-protocols, including rs-harness, ts-harness, py-harness, semantic-agent-hook installs, compact semantic search flow, and non-JSON agent command guidance.
---

# Agent Semantic Protocols

## Rules

- Choose the bin from the project language: Rust uses `rs-harness`, TypeScript uses `ts-harness`, Python uses `py-harness`.
- Start with `<bin> agent guide .` when unsure; it prints the provider-owned command menu.
- Do not add `--json` during agent exploration. `--json` is only for schema tests, validators, receipts, or IDE integrations.
- `semantic-agent-hook install --client codex .` installs hooks, profiles, and this skill at `.agents/skills/agent-semantic-protocols/SKILL.md`.

## Command Shapes

- Map the project: `<bin> search prime --view seeds .`
- Resolve an owner: `<bin> search owner <owner-path> --view seeds .`
- Search local text with tests: `<bin> search text <term> owner tests --view seeds .`
- Search external API/deps: `<bin> search deps <dep[/subpath][@version][::api]> .`
- Pipe candidate lines: `rg -n '<term>' src tests | <bin> search ingest --view seeds .`
- Check changed work: `<bin> check --changed .`

Rust owner and ingest accept extra scopes:

```sh
rs-harness search owner src/lib.rs items --view seeds .
rg -n 'HookDecision' src tests | rs-harness search ingest items tests --view seeds .
```

## Flow Examples

1. Implement a TypeScript feature around a known symbol:

```sh
ts-harness agent guide .
ts-harness search prime --view seeds .
ts-harness search text runCodexAgentHook owner tests --view seeds .
ts-harness search owner src/cli/agent-hooks.ts --view seeds .
ts-harness check --changed .
```

Use the `owner` output to choose the edit file. Use the `tests` seeds before editing.

2. Find Rust API usage before changing behavior:

```sh
rs-harness agent guide .
rs-harness search deps tokio::spawn public-api --view seeds .
rs-harness search text tokio::spawn tests --view seeds .
rs-harness search owner src/runtime.rs items --view seeds .
```

Use `deps` for external API facts. Use `owner` only after a real owner path appears.

3. Understand a Python implementation path:

```sh
py-harness agent guide .
py-harness search prime --view seeds .
rg -n 'Session' src tests | py-harness search ingest --view seeds .
py-harness search owner src/client.py --view seeds .
py-harness check --changed .
```

Use `rg` or `fd` to collect candidates, then let `py-harness` rank owners/tests.

## Combination Query Examples

1. Same TypeScript question, multiple names:

```sh
ts-harness search text --query-set runCodexAgentHook --query-set permissionDecision owner tests --view seeds .
```

Use this when both terms describe the same hook decision path.

2. Same Rust concept, type plus field:

```sh
rs-harness search text --query-set HookDecision --query-set permissionDecision tests --view seeds .
```

Use this when one answer should cover both aliases. Follow with `rs-harness search owner <path> items --view seeds .`.

3. Same Python API, import name plus method name:

```sh
py-harness search text --query-set requests.Session --query-set Session.request owner tests --view seeds .
```

Use this when API naming varies across imports/callsites.

Do not use query-set for independent axes. Run these separately and synthesize after reading compact outputs:

```sh
ts-harness search deps playwright::APIRequestContext .
ts-harness search owner src/http/client.ts --view seeds .
ts-harness search tests src/http/client.ts --view seeds .
```
