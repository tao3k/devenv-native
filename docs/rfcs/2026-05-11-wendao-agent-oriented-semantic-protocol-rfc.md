---
type: knowledge
title: "RFC: Agent-Oriented Semantic Protocol (AOSP) and Native Harness Architecture"
category: "rfc"
status: "draft"
authors:
  - auditor_neuron
  - sovereign
created: 2026-05-11
tags:
  - rfc
  - aosp
  - lsp-evolution
  - native-parsers
  - agentic-reasoning
  - cloud-context
---

# Agent-Oriented Semantic Protocol (AOSP) and Native Harness Architecture

## 1. The Paradigm Shift: From Human to Agent

For a decade, the Language Server Protocol (LSP) has been the industry standard. However, LSP was designed with a fundamental constraint: its target audience is a **Human Developer**, optimized for rendering local UI elements.

In 2026, the primary consumer of code semantics is the **Local AI Agent** (e.g., Gemini CLI, Claude Code, Codex). These agents perform reasoning, search, and generation locally, but they require deep structural context that local machines cannot efficiently compute for massive repositories.

This RFC proposes the **Agent-Oriented Semantic Protocol (AOSP)**. AOSP shifts the paradigm by separating **Local Inference** from **Cloud-Native Context Generation**.

## 2. The Core Moat: Native Parsers over Tree-Sitter

Generic Abstract Syntax Tree (AST) tools like `tree-sitter` are superficial. They lack the ability to comprehend deep, language-specific semantics (macro expansions, dynamic typings).

Our cloud backend abandons generic tools. Through language-specific harnesses (`python-lang-project-harness`, `rust-lang-project-harness`), we execute **compiler-level, native semantic parsing**. An Agent utilizing AOSP receives the exact same semantic graph that the native compiler sees, enabling flawless intent-based reasoning.

## 3. The Local-Inference / Cloud-Context Architecture

AOSP mandates a strict split: The "Brain" (LLM) runs locally, while the "Oracle" (Native Parsers & Ontology) runs in the cloud. They communicate via Arrow Flight.

### 3.1 The Local AOSP SDK (For Agentic Tools)

We will not build another IDE. Instead, we provide an **AOSP SDK** designed to be embedded directly into local agentic tools (Gemini CLI, Codex, etc.).
This SDK acts as the bridge, inheriting the best of legacy LSP:

- **Incremental VFS Sync**: Sending only code diffs (keystrokes, file saves) to the cloud to maintain a synchronized remote state with near-zero bandwidth.
- **Query Interface**: Allowing the local Agent to dispatch complex structural queries to the cloud.

### 3.2 The Stateful Cloud Context Engine (The Oracle)

The cloud infrastructure performs **no LLM inference**. It is a pure, high-performance semantic calculator.

1.  **Hot-Patching**: Upon receiving a diff from the local SDK, the specific `*-lang-project-harness` instantly updates its in-memory native AST.
2.  **Graph Recalculation**: The global Semantic Ontology Graph is updated via Julia matrix operations.
3.  **Semantic Responding**: When the local Agent queries the cloud, the cloud returns a rich, structured **Semantic Subgraph Payload** (not just a file line number).

## 4. The Agentic Self-Healing Loop via Native Harnesses

A critical differentiator of AOSP is its native integration with specialized policy engines like `rust-lang-project-harness` (and future equivalents like `python-lang-project-harness`).

Traditional linters (e.g., Clippy) output verbose, human-centric text. Our native harnesses are designed explicitly for agents, exposing compact, deterministic policy feedback (e.g., `AGENT-*` rules, `@ path:line:column`, `fix:`, and `Contract:`).

### 4.1 Cloud-Enforced Structural Policy

Under AOSP, local projects do not necessarily need to configure these harnesses manually via `dev-dependencies`.
As the local client pushes AST diffs to the cloud, the Cloud Oracle invisibly executes the full suite of native harness checks against the global graph. It detects architectural drift (e.g., `lib.rs` bloating, primitive obsession in public APIs, broken dependency boundaries) that local compilers ignore.

### 4.2 The Self-Healing Payload

When a violation occurs, the cloud returns a highly structured `harness_contracts` array within the AOSP Payload:

```json
{
  "harness_contracts": [
    {
      "rule_id": "AGENT-STRUCT-003",
      "severity": "Blocking",
      "location": "@ src/auth.rs:15:4",
      "fix_directive": "Extract implementation into a separate module to prevent lib.rs bloating."
    }
  ]
}
```

### 4.3 Autonomous Resolution

Upon receiving this payload, the local LLM Agent intercepts the violation before alerting the human developer. Because the feedback is a strict, programmatic contract rather than a conversational suggestion, the agent can autonomously refactor the code (e.g., creating a new module and migrating the function) and re-submit the diff. This creates a zero-friction, autonomous self-healing loop that continuously maintains codebase architecture at a machine-enforced standard.

## 5. Why AOSP Supersedes LSP for Agents

When a local Agent needs context to complete a task, standard LSP provides insufficient, noisy data (e.g., thousands of raw text references).

Under AOSP, the Agent uses the SDK to query the cloud.

- _Query_: "What are the downstream impacts of modifying the `Authenticator` trait?"
- _Cloud Response_: A structured JSON/Arrow payload detailing the exact call graph, trait implementors, and specific RFC violations stored in the `wendao-episteme` ontology.

The local LLM consumes this high-density structural data to generate accurate code, entirely eliminating the latency of uploading code to cloud LLMs and the inaccuracy of local vector searches.

## 5. Conclusion

AOSP represents the evolutionary successor to LSP. By isolating heavy native parsing and graph calculations in the cloud, and exposing an SDK for local AI agents to consume this data via zero-copy Arrow Flight, we empower local LLMs with global, compiler-level codebase comprehension without compromising local execution speeds or privacy.
