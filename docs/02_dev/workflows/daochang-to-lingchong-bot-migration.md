---
type: knowledge
metadata:
  title: "Daochang to Lingchong Bot Migration"
---

# Daochang to Lingchong Bot Migration

Telegram and Discord bot runtime ownership has moved out of this repository.
The main workspace is now responsible for Wendao, kernel, search, runtime,
memory, and shared contract surfaces. Bot/channel runtime work belongs in the
external `lingchong-bot` repository.

## Boundary

- This repository owns Wendao and shared Xiuxian kernel crates.
- `lingchong-bot` owns Telegram and Discord channel runtime behavior.
- Bot integrations should call Wendao through gateway or client boundaries.
- New bot runtime code must not be added back into this workspace.

## Cutover Notes

- The `xiuxian-daochang` workspace crate has been removed from the main Rust
  workspace.
- Agent runtime schemas consumed by Wendao now live under
  `packages/rust/crates/xiuxian-wendao/resources/agent/`.
- Shared Omega and memory gate contract types live in `xiuxian-types`.
- Non-bot process helpers formerly under `scripts/channel/` have moved to
  `scripts/runtime/`; bot launcher and probe scripts are no longer active in
  this workspace.

## Validation

After migration edits, validate the main repo with:

```bash
direnv exec . cargo metadata --no-deps --format-version 1
direnv exec . cargo check -p xiuxian-wendao --lib
direnv exec . cargo check -p xiuxian-logging --lib
direnv exec . cargo check -p xiuxian-llm --lib
```
