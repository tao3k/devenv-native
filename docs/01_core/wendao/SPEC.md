---
type: knowledge
title: "Xiuxian-Wendao: Semantic Knowledge & Skill Hub"
category: "core"
tags:
  - wendao
  - graph
  - skill-promotion
  - vfs
metadata:
  title: "Xiuxian-Wendao: Semantic Knowledge & Skill Hub"
---

# Xiuxian-Wendao: Semantic Knowledge & Skill Hub

`xiuxian-wendao` is the neural database of the CyberXiuXian OS. It integrates universal knowledge graph indexing with deep, hierarchical awareness of Agent Skills.

## 1. Multi-Format VFS Addressing (wendao://)

### 1.1 Zero-Copy & String Interning

Wendao employs an aggressive memory optimization strategy to support high-concurrency reasoning:

- **Protocol**: `wendao://skills/<semantic_name>/references/<entity_name>`
- **Support**: Seamless resolution across Markdown, TOML (Flows), and Jinja2 (Templates).
- **Security**: Strict path normalization prevents directory traversal attacks during resource resolution.

## 2. Tiered Semantic Discovery & Skill Promotion

The indexer employs a 2-layer discovery model to maximize both search breadth and domain depth.

### 2.1 Layer 1: Universal Link Extraction

Every document in the repository is parsed for Obsidian-style WikiLinks (`[[...]]`) and embeds (`![[...]`). These are automatically converted into typed relations in the Knowledge Graph.

### 2.2 Layer 2: Skill Promotion (Hierarchical Awareness)

When the indexer identifies a directory containing **`SKILL.md`**, the directory is "promoted" to a **Skill Entity**.

- **Deep Integration**: Wendao consumes parser-owned `SKILL.md` metadata to
  extract structured fields such as intents and routing keywords.
- **Identity Decoupling**: The logical `name` in the metadata overrides the physical crate ID, providing a stable semantic namespace.

## 3. Relational Logic

Wendao automatically establishes foundational graph relations during skill registration:

- **`[:CONTAINS]`**: Skill -> Tool (Executable scripts).
- **`[:REFERENCES]`**: Skill -> Reference Assets (Personas, Templates).
- **`[:RELATED_TO]`**: Tool -> Keyword (Concept entities).

## 4. Integration Status

- **VFS Resolver**: Fully Operational.
- **Skill Promotion**: Fully Operational.
- **Graph Proxy**: Fully Operational (Phase 11).
- **Full-Chain Mapping**: Fully Operational (Phase 14).
