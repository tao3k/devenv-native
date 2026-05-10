---
type: knowledge
title: "RFC: Native Parser-Driven Agentic Retrieval"
category: "rfc"
status: "draft"
authors:
  - auditor_neuron
  - sovereign
created: 2026-05-11
tags:
  - rfc
  - agentic-rag
  - native-parsers
  - arrow-flight
  - progressive-retrieval
---

# Native Parser-Driven Agentic Retrieval

## 1. Vision and Motivation

The industry standard for Retrieval-Augmented Generation (RAG) relies on probabilistic vector embeddings or generic Abstract Syntax Tree (AST) tools like `ast-grep` or `tree-sitter`. While useful, generic AST tools lack deep, language-specific semantic context (e.g., Rust macro expansion, Python dynamic typings, or specialized Markdown metadata hierarchies).

This RFC establishes the `xiuxian-artisan-workshop` retrieval paradigm: **A Progressive Evidence Funnel powered by Language-Native Parsers interconnected via Arrow Flight.** We replace generic string/AST searching with deterministic, native semantic querying, driven by LLM agents.

## 2. The Native Parsing Infrastructure

Our architecture possesses a significant physical moat: we do not rely on a single, monolithic parser for the entire repository.

### 2.1 Language-Native Parsers

Instead of using a lowest-common-denominator tool, the system employs native parsers for respective domains:

- **Rust analyzes Rust**: Utilizing native Rust compiler frontends/macros for deep semantic understanding.
- **Python analyzes Python**: Leveraging native Python AST and introspection.
- **Julia analyzes Julia**.
- **Wendao Specialized Markdown Parser**: A hyper-specialized parser capable of understanding complex document indices, nested reading orders, and embedded ontology metadata (e.g., extracting precise `page index` structures).

### 2.2 The Arrow Flight Interconnect

These disparate, highly specialized native parsers do not operate in silos. They are unified across process boundaries using **Arrow Flight**. This ensures that the extracted semantic structures (whether from Python code or Markdown documents) are normalized into zero-copy Arrow memory layouts, allowing ultra-fast cross-language data exchange and indexing without serialization overhead.

## 3. The Precision CLI Tooling (The "Agent's Scalpel")

The system exposes these native parsing capabilities via a unified Command Line Interface (CLI). This CLI serves as a super-powered alternative to traditional `grep` or `ast-grep`.

It allows Sub-agents to execute highly specific, language-aware queries. For example, an agent does not search for the string "auth"; it commands the CLI to: _"Use the native Rust parser to extract the implementation details of the `Authenticator` trait within the `xiuxian-wendao-server` crate."_

## 4. The Progressive Retrieval Funnel (Agentic Workflow)

We reject the notion that deterministic search (`grep`/CLI) is dead. Instead, it is strategically repositioned. The retrieval workflow operates as a progressively narrowing funnel, avoiding the "infinite noise" of executing raw searches without context.

### Tier 1: Query Understanding & Ontology Navigation (The Compass)

When a vague intent is received, the LLM engages with the `wendao-episteme` Ontology. Through deductive reasoning, it translates the ambiguous query into structural hypotheses.

- _Output_: It generates specific "Evidence Keywords", target file paths, or domain classifications.

### Tier 2: Strategy Flow & Graph Pruning (The Map)

The Julia strategy layer takes the ontological clues and prunes the global semantic graph, further narrowing the physical scope of the investigation.

### Tier 3: Deterministic Native Strike (The Scalpel)

Armed with precise clues from Tiers 1 and 2, the Sub-agent delegates execution to the **Native Parser CLI**.

- Because the scope and intent are now laser-focused, the CLI executes native parsing over the targeted files.
- _Outcome_: It returns 100% deterministic, high-signal code blocks, Markdown sections, or page indices.

## 5. Conclusion

By delegating deep structural understanding to Language-Native Parsers, standardizing data flow via Arrow Flight, and orchestrating retrieval through a progressive LLM-guided funnel, this architecture eliminates the hallucination risks of vector RAG and the semantic blindness of generic `ast-grep`. The LLM provides the deductive strategy; the Native CLI provides the surgical physical extraction.
