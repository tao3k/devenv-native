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

Zhenfa keeps a JSON-RPC compatible envelope for external gateway calls and testable transport boundaries.

The schema contract covers:

1. `jsonrpc`, `id`, `method`, `params`, and optional trace metadata.
2. stable error codes and `ZhenfaError` mapping.
3. result payload transport for already-rendered strings or typed gateway DTOs.

Zhenfa does not generate model-facing tool schemas and does not own LLM function-calling registration. Callers that need model/tool policy must adapt direct Zhenfa functions in their own runtime layer.

## Request Envelope

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
    "session_id": "session-42",
    "trace_id": "span-5678"
  }
}
```

## Response Envelope

```json
{
  "jsonrpc": "2.0",
  "id": "req-uuid-1234",
  "result": "<hit id=\"task_01\" score=\"0.95\">Write tests</hit>"
}
```

## Error Envelope

```json
{
  "jsonrpc": "2.0",
  "id": "req-uuid-1234",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "details": "The limit parameter must be <= 100."
    }
  }
}
```
