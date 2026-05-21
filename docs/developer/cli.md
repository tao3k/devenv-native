---
type: knowledge
title: "CLI Developer Guide"
category: "developer"
tags:
  - developer
  - cli
saliency_base: 6.3
decay_rate: 0.04
metadata:
  title: "CLI Developer Guide"
---

# CLI Developer Guide

Core user-facing commands are documented in [CLI Reference](../reference/cli.md).

## Current State

The historical Python `omni` CLI command tree has been removed.

Python no longer ships:

1. skill command groups
2. route/sync/reindex/db command groups
3. local runner daemons
4. Python agent/gateway entrypoints

The retained Python role is narrow:

1. Arrow Flight transport/client helpers in `packages/python/wendao-core-lib`
2. thin schema/config/test helper surfaces

Operational Wendao and kernel entrypoints live in Rust. Telegram and Discord
bot runtime entrypoints live in the external `lingchong-bot` repository.

## Developer Rule

Do not add new Python CLI surfaces that recreate local runtime ownership.
New operational commands belong in Rust unless they are strictly thin transport
or helper wrappers.
