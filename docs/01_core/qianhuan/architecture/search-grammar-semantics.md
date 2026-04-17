---
type: knowledge
title: "Qianhuan-Wendao Search Grammar & Semantics"
category: "architecture"
tags:
  - qianhuan
  - wendao
  - search-grammar
  - semantics
saliency_base: 8.5
decay_rate: 0.01
metadata:
  title: "Qianhuan-Wendao Search Grammar & Semantics"
---

# Qianhuan-Wendao Search Grammar & Semantics

This document defines how the system uses the **Wendao Query Grammar** to precisely retrieve configurations (Personas and Templates) managed within Markdown files.

## 1. Unified Search Interface

All configuration lookups from `xiuxian-qianhuan` or `xiuxian-qianji` are routed through the same `wendao.search` endpoint provided by the [[docs/01_core/zhenfa/SPEC.md|Zhenfa Gateway]].

The grammar supports three levels of precision: **Exact**, **Scoped**, and **Semantic**.

## 2. Precision Levels

### 2.1 Level 1: Exact ID Resolution ($O(1)$)

Used when the specific ID of a Persona or Template is already known (e.g., in a Qianji workflow node).

- **Syntax**: `id:<unique_id>`
- **Example**: `id:agenda_steward`
- **Result**: Returns the specific `Entity` node matching that ID.
- **Backend Path**: Bypasses the link-graph and vector database; fetches directly from Valkey.

### 2.2 Level 2: Scoped Metadata Filtering

Used to find assets belonging to a specific domain or of a specific type.

- **Syntax**: `type:<config_type> [metadata_key]:<value>`
- **Example**: `type:persona domain:zhixing`
- **Logic**: Filters entities where the `type` attribute in the Markdown HTML comment was `persona` and the `domain` was `zhixing`.
- **Use Case**: "Show me all available personas for the Zhixing domain."

### 2.3 Level 3: Semantic Discovery

Used when the ID is unknown, but the _intent_ or _voice_ is described.

- **Syntax**: `related_to:<concept> type:persona`
- **Example**: `related_to:discipline type:persona`
- **Logic**: Uses Wendao's PPR (Personalized PageRank) and Vector Similarity to find Persona nodes that are semantically close to "discipline".
- **Result**: Might return the `Strict Teacher` persona because its background text mentions "discipline" and "sternness".

## 3. The Injection Payload Contract

When a query is marked as a **Configuration Request** (either by the `type:` directive or a specific header), Wendao applies the following **Stripping Logic**:

1. **Header Extraction**: It identifies the Heading that owns the matched ID.
2. **Code Block Slicing**: It extracts the _first_ fenced code block under that heading.
   - If `type:persona`, it expects `toml`.
   - If `type:template`, it expects `jinja2`.
3. **Direct Return**: The response `result` string contains the _raw content_ of that code block, ensuring the LLM or Qianhuan receives data without JSON/Markdown framing.

## 4. Best Practices for Authoring

To ensure your Markdown-managed configurations are "searchable" and "indexable":

1. **Anchor with HTML Comments**: Always place `<!-- id: "...", type: "...", domain: "..." -->` immediately below your H2 heading.
2. **Use Clear Headings**: Heading text is used for semantic vector indexing. `## Persona: Strict Teacher` is better than `## P1`.
3. **Isolate Code Blocks**: Ensure only one configuration-relevant code block exists per ID scope to avoid ambiguity during extraction.

## 5. Summary Table

| Intent                         | Query Grammar                   | Precision            |
| :----------------------------- | :------------------------------ | :------------------- |
| **Booting a specific persona** | `id:steward_v2`                 | 100% (Exact)         |
| **Listing all templates**      | `type:template`                 | High (Filter)        |
| **Finding a stern voice**      | `related_to:stern type:persona` | Semantic (Discovery) |

## 6. The Semantic-to-ID Resolution Layer

To bridge the gap between fuzzy user intent (e.g., "I want a stern teacher") and precise internal execution ($O(1)$ ID lookup), Wendao implements a **Two-Stage Resolution Pipeline**.

### 6.1 Stage 1: Intent Mapping (The "Winner" Selection)

When a user provides keywords or tags instead of a raw ID, the Rust host (or the Omega Deliberation Engine) performs a **Selection Query**:

1.  **Query**: `tag:voice:stern type:persona limit:3`
2.  **Wendao Action**: Uses Vector similarity and PPR to find the top 3 personas matching "stern".
3.  **Internal Decision**:
    - **Auto-Winner**: If the top result has a confidence score > 0.9, the system automatically selects its `id`.
    - **LLM Choice**: If results are ambiguous, the LLM (Cognitive Interface) is shown the candidates and asked to pick one.

### 6.2 Stage 2: Precise Execution (O(1) Handover)

Once the "Winner ID" (e.g., `strict_teacher_v2`) is resolved from Stage 1, the system passes this ID to the [[docs/01_core/wendao/architecture/id-resolution-mechanism.md|ID Resolution Mechanism]].

From this point forward, the execution is **100% deterministic**. The system no longer "searches"; it "fetches" the exact code/template from memory using the $O(1)$ index.

## 7. Best Practice: Deterministic Slugs as IDs

To make IDs easier for humans to manage within Markdown, we recommend using **Semantic Slugs** instead of random numbers:

- ❌ **Bad ID**: `<!-- id: "12345-abcde", type: "persona" -->`
- ✅ **Good ID**: `<!-- id: "agenda-steward-polite", type: "persona" -->`

By authoring with semantic slugs, the "ID" itself becomes a human-readable keyword that functions as a high-speed primary key in the Rust runtime.

## 8. Summary Table Table update
