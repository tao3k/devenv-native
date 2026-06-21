---
type: knowledge
title: "HTTP Client Invocation Paradigm"
category: "architecture"
tags:
  - zhenfa
  - http
  - invocation
saliency_base: 8.0
decay_rate: 0.02
metadata:
  title: "HTTP Client Invocation Paradigm"
---

# HTTP Client Invocation Paradigm

By introducing the `xiuxian-zhenfa` matrix, we drastically alter how the LLM "Action Compiler" and the `xiuxian-daochang` host interact with domain tools.

## The Legacy Native Paradigm

Previously, tools like `wendao.search` or `agenda.view` required the `xiuxian-daochang` binary to compile against the full Rust source code of `xiuxian-wendao` and `retired Zhixing crate`. The host had to manage massive dependency trees, and any change required a full workspace recompile.

## The Zhenfa Paradigm

Under the Zhenfa architecture, the `xiuxian-daochang` tool registry becomes incredibly thin.

1. **Thin Client Wrappers**: The agent's native tools are refactored to be pure HTTP clients (e.g., using `reqwest`).
2. **Search-Driven Invocation**: When the LLM decides to invoke a tool like `wendao.search(query="...")`, the agent forwards this query via a `POST` request to the **Zhenfa** gateway.
3. **Internal Routing**: Zhenfa hands the query to the **Wendao Search Engine**, which resolves the query against the **Valkey** storage layer and returns rendered results.
4. **Omni-Channel Availability**: Because the tools are now exposed over HTTP, any language or interface (Python scripts, Web UI, external clients) can utilize the exact same high-performance Rust backend without navigating PyO3 FFI bindings.

## Hot-Reloading

This paradigm naturally supports hot-reloading. For instance, sending a domain-specific reload request to the Zhenfa gateway can refresh context templates across the network without restarting the agent's conversational state.
