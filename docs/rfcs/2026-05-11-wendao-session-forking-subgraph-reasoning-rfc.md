---
type: knowledge
title: "RFC: Session Forking and Sub-Graph Reasoning for LLM Agents"
category: "rfc"
status: "draft"
authors:
  - auditor_neuron
  - sovereign
created: 2026-05-11
tags:
  - rfc
  - agentic-memory
  - context-management
  - session-forking
  - sub-graph-reasoning
---

# Session Forking and Sub-Graph Reasoning for LLM Agents

## 1. Vision and Motivation

As AI coding assistants (like Claude Code, Codex, or Cursor) tackle increasingly complex, repository-scale tasks, they encounter a critical limitation: **Context Window Degradation**.

Traditional LLM interactions rely on a linear session history. As an agent reads multiple files, explores dead ends, and iterates on code, the context window becomes polluted with irrelevant "noise." This noise dilutes the model's attention mechanism, leading to severe hallucinations, loss of focus, and logical errors in later reasoning stages.

This RFC proposes a paradigm shift from linear session management to **Session Forking and Sub-Graph Reasoning**, applying Git-like branching concepts to LLM context orchestration.

## 2. The Architecture: Context as a Directed Acyclic Graph (DAG)

We abandon the concept of a single, continuous chat thread. Instead, the agent's reasoning process is managed as a tree of isolated context states.

### 2.1 The Semantic Checkpoint

When an agent reaches a state of clear comprehension regarding a specific domain (e.g., "I have successfully mapped the database schema" or "I understand the native AST structure of `auth.rs`"), the system creates a **Semantic Checkpoint**.
This checkpoint takes a snapshot of the _pure, high-signal context_ (the necessary code snippets, ontology rules, and the agent's summarized understanding), discarding all the conversational trial-and-error that led to it.

### 2.2 Session Forking

When the agent begins a new specific coding task (e.g., "Implement the JWT validation function"), it **does not append** to the long, polluted historical session.

Instead, the orchestrator **Forks** a new, isolated LLM session. This new session is seeded _only_ with the specific Checkpoint data relevant to the task.

### 2.3 Sub-Graph Reasoning (The Sandbox)

The agent now operates within a highly focused **Sub-Graph**.

- **Absolute Focus**: The LLM's context contains only the exact code it needs to edit and the exact rules it must follow.
- **Hallucination Elimination**: Because unrelated codebase history is physically absent from the context window, the model cannot hallucinate variables or logic from other domains.

## 3. Implementation via Org-Mode and Non-Intrusive SDK

Crucially, this architecture is designed to augment, not replace, frontier agents (Claude, Codex). We act as the context orchestrator beneath them.

The **Org-mode** structure defined in previous RFCs serves as the physical map for this Session Graph:

```org
* EPIC: Refactor Authentication
  * DONE [Checkpoint: Auth Domain Map]
    :PROPERTIES:
    :CONTEXT_HASH: x9f2a
    :END:

    ** DOING [Fork: from x9f2a] Task: Rewrite verify_token
       # The LLM session here only contains context x9f2a and this specific task.

    ** TODO [Fork: from x9f2a] Task: Update User Model
       # An entirely separate session, immune to errors made in the verify_token task.
```

## 4. Conclusion

By treating LLM context as a manipulable, branchable graph rather than an immutable, linear log, we solve the fundamental problem of context degradation. Session Forking allows us to leverage the immense power of generic frontier models while maintaining the absolute surgical precision required for enterprise software engineering.
