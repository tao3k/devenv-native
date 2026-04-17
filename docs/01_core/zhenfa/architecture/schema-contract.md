---
type: knowledge
title: "Zhenfa JSON-RPC Schema Contract"
category: "architecture"
tags:
  - zhenfa
  - schema
  - json-rpc
  - api
saliency_base: 8.5
decay_rate: 0.01
metadata:
  title: "Zhenfa JSON-RPC Schema Contract"
---

# Zhenfa JSON-RPC Schema Contract

To prevent HTTP payload fragmentation across the `wendao`, `qianhuan`, and `qianji` endpoints, the `xiuxian-zhenfa` matrix enforces a strict, unified envelope for all requests and responses.

We adopt a lightweight JSON-RPC 2.0 inspired schema to ensure standard error handling, trace propagation, and deterministic LLM tool calling.

## 1. The Request Envelope

Every POST request hitting the Zhenfa gateway must conform to this schema:

```json
{
  "jsonrpc": "2.0",
  "method": "wendao.search",
  "id": "req-uuid-1234",
  "params": {
    "query": "agenda date:this_week status:open",
    "limit": 10
  },
  "meta": {
    "session_id": "telegram-chat-42",
    "trace_id": "span-5678"
  }
}
```

### 1.1 Field Definitions

- `method`: The fully qualified action name (e.g., `wendao.search`, `qianhuan.render`, `qianji.trigger`). This perfectly aligns with the function names exposed to the LLM.
- `params`: An arbitrary JSON object specific to the `method`. This is strictly validated by the underlying Rust Axum handler using `serde`.
- `meta`: (Optional) Propagation headers for OpenTelemetry and cross-agent context tracking.

## 2. The Response Envelope

### 2.1 Success Response (LLM Token Optimized - The Stripping Layer)

When returning data intended directly for the LLM's context window (e.g., search results), returning deeply nested JSON arrays is an anti-pattern. JSON braces, quotes, and commas consume massive amounts of unnecessary tokens and induce formatting hallucinations.

Instead, the Native `ZhenfaTool` implementation must return a **pre-rendered, raw string** (preferably XML-Lite) that is perfectly formatted for the LLM. The HTTP layer simply wraps this string in the `result` field.

```json
{
  "jsonrpc": "2.0",
  "id": "req-uuid-1234",
  "result": "<hit id=\"task_01\" score=\"0.95\">\nWrite tests\n</hit>\n<hit id=\"task_02\" score=\"0.88\">\nUpdate docs\n</hit>",
  "metrics": {
    "execution_ms": 1.5,
    "cache_hit": true
  }
}
```

_Note: The HTTP envelope remains JSON to ensure the network layer can parse it easily, but the actual payload given to the LLM is the lean string inside `result`. For native in-process calls, the JSON envelope is entirely bypassed, and the `result` string is passed directly via memory._

### 2.2 Error Response (The Standard Taxonomy)

When an error occurs (e.g., LLM hallucinates a parameter, or Wendao cannot find an index), the response MUST follow this structure to allow the LLM or 修仙道场 to execute a retry or fallback strategy.

```json
{
  "jsonrpc": "2.0",
  "id": "req-uuid-1234",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "details": "The 'limit' parameter must be <= 100."
    }
  }
}
```

## 3. Tool Descriptions & Skill Manuals

While `xiuxian-zhenfa` can use tools like `utoipa` to generate OpenAPI schemas for programmatic routing, **we DO NOT dump raw JSON Schema into the LLM's context window.**

JSON Schema definitions are token-heavy and distract the model. Instead:

1. **Tool Definition (`xiuxian-daochang`)**: The agent reads the strict schema to register the standard function calling interface (e.g., `wendao.search(query)`).
2. **Skill Manual (The Prompt)**: The actual "instruction manual" injected into the LLM via `Qianhuan` (see `QH-08 Zero-Hardcoding`) is a lean, human-readable Markdown or XML document explaining the query syntax, not a massive JSON structure.

## 4. End-to-End Execution Flow (Dual-Mode)

To clarify how the LLM interacts with Zhenfa without being polluted by JSON wrappers, here is the lifecycle of a query like _"What did I do this week?"_ (For the full theoretical justification, see [[docs/99_llm/architecture/token-economics-and-attention.md|Token Economics & Formatting Hallucination]]).

### 4.1 Native Execution Path (In-Process, The "Codex" Path)

This is the primary path used by `xiuxian-daochang` to achieve zero-latency execution.

1. **Session Boot (Skill Injection)**:
   - `Qianhuan` injects the Persona and the **Wendao Skill Manual** (Markdown/XML explaining how to use `wendao.search` with constraints).
2. **Action Compilation (LLM Trigger)**:
   - The LLM reasons: _"I need to search the agenda for this week."_
   - It outputs a tool call: `wendao.search(query="agenda date:this_week")`.
3. **Registry Dispatch (Native Rust Call)**:
   - The `xiuxian-daochang` host intercepts the tool call.
   - Instead of making an HTTP request, it directly queries the in-memory `ZhenfaRegistry` for the `wendao.search` trait object.
   - It calls `tool.call_native(ctx, args).await`.
4. **Execution & Stripping (Zero-Copy)**:
   - Wendao executes the PPR search natively.
   - Wendao's `ZhenfaTool` implementation formats the raw hits directly into a clean, **Stripped XML-Lite string**.
5. **Context Assimilation**:
   - The native call returns the String (no JSON parsing needed).
   - The `xiuxian-daochang` appends the string as a `role="tool"` message to the LLM's context window.

### 4.2 Matrix Execution Path (HTTP, External Integration)

Used when a Python script or external system calls the gateway.

1. **Network Transit (JSON Envelope)**:
   - The external client wraps the request in the **Zhenfa JSON-RPC Request Envelope** (Section 1) and sends a `POST` over HTTP to `xiuxian-zhenfa`.
2. **Axum Gateway Conversion**:
   - `xiuxian-zhenfa` parses the JSON, looks up the tool in the `ZhenfaRegistry`, and calls `tool.call_native(ctx, args).await`.
3. **Response Envelope Wrapping**:
   - The native tool returns the stripped string.
   - Axum wraps this string into the **Zhenfa JSON-RPC Response Envelope** (Section 2.1) and returns it over HTTP.
4. **The Client Stripping Layer**:
   - The external client MUST strip away the `jsonrpc`, `id`, and `metrics` wrappers and pass ONLY the `result` string to its respective LLM.
