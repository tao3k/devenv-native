---
type: knowledge
title: "Qianji Architecture: Multi-Persona Context Window Management"
category: "architecture"
tags:
  - qianji
  - multi-agent
  - context-isolation
  - LLM
saliency_base: 7.5
decay_rate: 0.02
metadata:
  title: "Qianji Architecture: Multi-Persona Context Window Management"
---

# Qianji Architecture: Multi-Persona Context Window Management

A critical architectural decision in Qianji is determining _when_ to isolate the LLM state and _when_ to append to it.

Academic research in 2024-2025 on Multi-Agent Debate (MAD) highlights fatal flaws when agents share conversational context: **Context Contamination**, **Sycophancy** (agents agreeing with each other instead of reasoning independently), and **Persona Drift** (agents losing their designated role parameters).

To solve this, Qianji enforces strict **Memory Boundaries** through distinct execution modes.

## Scenario A: Multi-Persona Adversarial Debate (Isolated & Ephemeral Windows)

**Use Case:** Validation, Auditing, Critique (e.g., `Agenda Steward` vs `Strict Teacher`).

- **Mechanism:** Each node evaluation spins up a **brand-new, ephemeral HTTP request**.
- **Data Flow (Strict Quarantine):** The annotation engine generates a completely distinct `InjectionSnapshot` for each node based on its unique `persona_id`. The underlying persona instructions and node-specific thought processes are strictly quarantined.
- **Contextual Grounding (The `xiuxian-window` Bridge):** To prevent the isolated agent from becoming "amnesiac" to the user's ongoing conversation, Qianji injects the _read-only, sanitized recent chat history_ (provided by `xiuxian-window`) into the `<working_history>` block of the node's `InjectionSnapshot`. This allows the agent to understand the user's immediate intent without being contaminated by another agent's hidden CoT.
- **Node Handoff:** Only the _structured XML output_ (e.g., `<agenda_draft>`) of Node A is passed as an _input variable_ to Node B's J2 template.
- **Research Alignment:** This guarantees the "Strict Teacher" never sees the "Agenda Steward's" internal Chain-of-Thought (CoT), completely eliminating real-time social influence and preventing Sycophancy, while remaining contextually aware of the user's request.

## Scenario B: Single-Persona Tool Execution (Appended & Continuous Windows)

**Use Case:** Multi-step tool usage, complex reasoning loops by a single agent (e.g., standard ReAct loop).

- **Mechanism:** The node sets `execution_mode = "appended"` and persists history through a defined `history_key`.
- **Data Flow:** The host runtime can append tool outcomes or prior snapshots into the `history_key`, and downstream appended nodes continue from that state. This allows the LLM to maintain a coherent "train of thought" and react to sequential tool outputs without losing its initial grounding.

## Scenario C: Single-Window Multi-Persona Debate (Economic Mode)

While Isolated Windows (Scenario A) provide maximum Persona purity, they incur a heavy **Token Cost Penalty** because every distinct persona request must duplicate the user's `working_history` and `wendao_context`. For cloud models (e.g., GPT-4o, Claude 3.5), this parallel isolation can quickly burn through token budgets.

To balance purity and economics, Qianji supports an **Economic Debate Mode (Role-Mix Profile)**:

- **Mechanism:** A single Qianji node invokes the LLM _once_.
- **Data Flow:** The annotation engine dynamically generates a `RoleMixProfile` (as defined in `injection-evolution.md`) that explicitly partitions a _single_ system prompt into multiple persona blocks using strict XML isolation.
- **XML-Enforced Debate Protocol:** To prevent the LLM from merging the personas, the output format is rigidly locked into an XML conversation structure. The LLM must "talk to itself" using explicit `<turn>` tags mapped to specific roles.

### Economic Mode Prompt Structure Example:

```xml
<persona_steering mode="multi_role_debate">
  <roles>
    <role id="agenda_steward">
      <objective>Propose the schedule.</objective>
      <input_focus>user_request</input_focus>
    </role>
    <role id="strict_teacher">
      <objective>Critique the steward's schedule.</objective>
      <input_focus>wendao_historical_carryover</input_focus>
    </role>
  </roles>
  <debate_protocol>
    You must simulate a rigorous debate between the roles. You MUST output your response exactly in this XML format:
    <debate_transcript>
      <turn role="agenda_steward">
        <thought_process>...</thought_process>
        <draft_agenda>...</draft_agenda>
      </turn>
      <turn role="strict_teacher">
        <critique>...</critique>
        <score>...</score>
      </turn>
      <turn role="agenda_steward">
        <!-- The steward's revised defense or updated draft based on the critique -->
        <revised_agenda>...</revised_agenda>
      </turn>
    </debate_transcript>
    <final_consensus>...</final_consensus>
  </debate_protocol>
</persona_steering>
```

**Trade-off:** The LLM is forced to "role-play" multiple entities sequentially within the same context window. It saves massive token costs (history is only sent once) and allows the LLM's internal attention mechanism to resolve the conflict. However, it relies heavily on the LLM's instruction-following capability to adhere strictly to the XML `<turn>` tags to prevent the personas from bleeding into one another (Sycophancy). This mode is ideal for highly capable frontier models.

## Concurrent Gathering

For isolated scenarios requiring multiple independent critics (e.g., a "Security Auditor" and a "Performance Auditor"), Qianji can evaluate their nodes concurrently using Rust's `tokio::spawn`, later joining their quarantined XML outputs at a terminal aggregation node.
