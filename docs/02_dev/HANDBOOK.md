---
type: knowledge
title: "How to Query and Locate Ingested Papers"
category: "how-to"
tags:
  - how-to
  - knowledge
saliency_base: 6.0
decay_rate: 0.05
metadata:
  title: "How to Query and Locate Ingested Papers"
---

# How to Query and Locate Ingested Papers

This guide describes the end-to-end flow: user asks for papers (for example
"multimodal document parsing" or "ingest-related papers"), and the agent uses
knowledge tools or CLI flows to find and cite them.

---

## Flow Overview

```
User: "Find me papers about multimodal document parsing / ingest-related papers"
        │
        ▼
Agent calls knowledge tools or CLI helpers
        │
        ├── knowledge.recall(query="multimodal document parsing paper", limit=5)
        │   → Returns chunks with content, source, score from vector store
        │
        └── knowledge.search(query="RAG anything ing paper", mode="hybrid")
            → Returns merged LinkGraph + vector results; vector hits are from ingested docs
        │
        ▼
Agent interprets results and answers user
        → "The paper you ingested (e.g. arXiv 2510.12323) is in the knowledge base.
           Relevant snippets: [content]. The matching document can be identified
           from its source metadata or citation id."
```

---

## Knowledge Tools to Use

| Tool                 | When to use                                                                                  | Example                                                |
| -------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| **knowledge.recall** | Semantic search over the vector store (ingested PDFs, markdown, etc.)                        | Query: "multimodal document parsing overview abstract" |
| **knowledge.search** | Hybrid (LinkGraph + vector) or keyword-only; good when you want both notes and ingested docs | Query: "document parsing ingest paper", mode: "hybrid" |

Both can surface content from an ingested PDF. Recall returns `content`, `source`, `score`; search returns merged results with `source` and reasoning.

### Action-based recall

For long content, a single `knowledge.recall` with default `chunked=True` runs preview → fetch → all batches in one call, which can time out and cause memory accumulation. Use one step per tool call:

1. **start** – `knowledge.recall(query="...", chunked=True, action="start")` → preview only, returns `session_id` and `batch_count` (no full fetch; avoids memory spike).
2. **batch** – `knowledge.recall(session_id="<from start>", action="batch", batch_index=0)` … then `batch_index=1`, etc. → each call lazy-fetches and returns one batch (no full state in memory).
3. **full_document** – `knowledge.recall(chunked=True, action="full_document", source="2601.03192.pdf")` → returns **all chunks** for that document, sorted by `chunk_index`. Use when you need the **complete paper with no omission** (semantic search returns top-N and may miss chunks).

Each batch response is small; the LLM reads slice by slice. This avoids memory accumulation and token limits.

---

## How to "Locate" the Paper

- **After ingest**: The PDF is chunked and stored in the knowledge vector
  store with metadata such as `source` and `title`.
- **In recall results**: The `source` field in the API may be the chunk ID (e.g. UUID) depending on the vector backend. To show the user "which paper" a snippet came from, the agent can:
  1. Use the **content** of the recalled chunks (for example a paper title,
     figure caption, or abstract sentence) to infer the document.
  2. If the system exposes source metadata, use that to cite the document by
     source identifier, title, or arXiv id.

So "locating" the paper means: run recall/search with a natural-language query about the topic, then report the matching snippets and, when available, the document path or arxiv id from metadata or context.

---

## Example: User Asks for "Document Parsing / Ingest-Related Papers"

1. **User**: "帮我找寻文档解析或 ingest 相关的论文" (or: "Find me papers about
   document parsing or ingest-related work.")

2. **Agent** calls knowledge tools:
   - `knowledge.recall(query="document parsing or ingest pipeline paper", limit=5)`
   - Optionally: `knowledge.search(query="document parsing ingest paper", mode="hybrid")`

3. **Tool call returns** (example):
   - Chunks such as a figure caption, abstract sentence, or method summary
     with high score.
   - Other snippets about multimodal analysis, ingestion pipelines, and
     retrieval systems.

4. **Agent answers user**:
   - "The knowledge base contains a paper that matches your request. Relevant
     excerpts: [paste content]. The document can be cited from its source
     metadata or arXiv id."

---

## Prerequisites

- The target paper (or its PDF) must already be **ingested** via `knowledge.ingest_document` (e.g. after downloading the PDF to a local path). Download first, then ingest.
- Vector store must be available (e.g. after `omni sync knowledge` or `knowledge.ingest_document`); then recall/search work without extra runtime setup.

## Prefer Built-In Tooling

Prefer direct tool calls or CLI paths such as `omni skill run ...` and `omni knowledge recall ...`. Do not assume a separate external tool server is available.

## CLI: Fast path vs full skill

- **Fast path (recommended for CLI)**: `omni knowledge recall "query" [--limit N] [--json]`  
  Uses only the foundation vector store and embedding; **typically under 2s**. No kernel/skill stack.
- **Full skill**: `omni skill run knowledge.recall '{"query":"..."}'`  
  Loads full kernel and all skills (30–45s cold start); use when you need fusion boost (LinkGraph, KG).

## Timeouts (knowledge.recall)

- **Embedding**: Query embedding is limited by `knowledge.recall_embed_timeout_seconds` (default **18**). If the embedding service is slow or unreachable, recall falls back to a hash-based vector so the request returns within the limit (with potentially lower relevance) instead of hitting a client-side timeout.
- **Tool execution**: If recall still times out, use CLI `omni knowledge recall "your query" --limit 5` or increase `knowledge.recall_embed_timeout_seconds` (e.g. 25).

---

## Limit vs preview vs full read

- **limit** = how many **items** to return (for accuracy list or batch size). Not for "how much content" to read. Use **preview** to confirm recall is right; use a **workflow** to read long content in chunks.
- **preview** (`recall(..., preview=True, snippet_chars=150)`): returns only title, source, score, and first N chars per result → use to **verify accuracy** before pulling full content.
- **Long content in chunks** (papers, manuals, long docs): Recalled content is usually long, so `knowledge.recall` **default** is the chunked workflow (preview → fetch → batches). Chunking is consumed in memory: feed `batches[i]` to the LLM in turn so each slice stays in context. Response includes `preview_results`, `batches`, `all_chunks_count`, `results`. Use `chunked=False` for single-call search only.

## Research workflow: use ingested content

To **research or analyze** any long ingested content (paper, manual, long doc):

1. **Default (chunked)**: Call `knowledge.recall(query="…")` → get `preview_results`, `batches`, `results`; use preview to confirm accuracy, then feed `batches[i]` to the LLM one batch per turn.
2. **Single-call**: `knowledge.recall(query="…", chunked=False, limit=N)` for one batch of full chunks (no workflow).
3. **If recall times out**: Run `omni knowledge recall "…" --json` locally, or increase `knowledge.recall_embed_timeout_seconds`.

---

## Summary

- **Query**: Use natural-language queries with `knowledge.recall` or `knowledge.search` (hybrid).
- **Locate**: Use returned content + metadata (and, when available, document path) to tell the user which paper the snippets came from.
- **End-to-end**: Ingest PDF → User asks for papers → Agent uses knowledge tools → Agent returns snippets and paper identity (path/arxiv id).

---

## Bot Runtime Migration

The former `xiuxian-daochang` Rust agent and Telegram/Discord channel runtime
has been decommissioned from this workspace. Bot runtime ownership now belongs
to the external `lingchong-bot` repository. This repository keeps Wendao,
Qianji, memory, LLM infrastructure, search, and shared kernel contracts.

Current validation should use the package-owned commands in this handbook and
the root `just` recipes. Do not add new `xiuxian-daochang` CLI, workspace, or
channel-runtime instructions here. Bot/channel operational docs should be added
to `lingchong-bot` instead.

For migration context, see [Daochang to Lingchong Bot Migration](workflows/daochang-to-lingchong-bot-migration.md).
