---
type: knowledge
title: "Xiuxian-Forge (炼魂): Autonomous Skill Evolution"
category: "core"
tags:
  - forge
  - evolution
  - memrl
  - synaptic-flow
metadata:
  title: "Xiuxian-Forge (炼魂): Autonomous Skill Evolution"
---

# Xiuxian-Forge (炼魂): Autonomous Skill Evolution

`xiuxian-forge` is the evolutionary engine of the CyberXiuXian OS. It enables the Agent to autonomously identify cognitive gaps and "refine" new specialized personas and methodologies.

## 1. The Three-Layer Forge Philosophy

1.  **The Trigger (Cognitive Pain)**: Detecting clusters of low Q-scores or repeated node failures via Valkey.
2.  **The Synthesis (Alchemical Refinement)**: Analyzing failure traces to identify semantic gaps and generating a new "Artisan Soul" using standardized templates.
3.  **The Mounting (Physical Manifestation)**: Dynamic injection of forged assets into the VFS and LinkGraph bus.

## 2. The Forge Trigger (Valkey Pain Detection)

The system identifies "Cognitive Dissonance" through a sliding window of audit scores.

### 2.1 Pain Detection Algorithm

- **Sliding Window**: Monitors the last 10 audit scores for a specific domain.
- **Critical Failure**: Any score $< 0.3$ increments the `fail_count`.
- **Consensus Threshold**: If the window average $< 0.4$ or `fail_count` $\ge 5$, a **Forge Ignition Signal** is emitted.

## 3. The Alchemical Synthesis Pipeline

The Forge engine does not operate in isolation; it utilizes the **Zhenfa Artery** to perform high-fidelity synthesis.

### 3.1 Trace Harvesting (via Native Wendao)

When a Forge Signal is emitted, the engine invokes the **`wendao.search`** native tool:

- **Query**: Filters for `outcome=failed` and specific `domain` keys in the audit stream.
- **Protocol**: Uses the **Native-First Dispatch** to retrieve raw synaptic traces with sub-millisecond latency.

### 3.2 Cognitive Washing (via ZhenfaTransmuter)

All harvested failure data passes through the **`resolve_and_wash`** interface:

- **Integrity**: Ensures the "DNA" of the failure (the XML blocks) is not corrupted.
- **Refinement**: Strips away non-essential noise to focus the LLM's "Soul-Forger" persona on the core logic gap.

### 3.3 Persona Mounting

The synthesized output (MD/TOML) is dynamically mounted into the skill runtime via the `SkillRuntimeResolver`, allowing the Agent to possess the new soul in the very next session.

## 4. The Soul-Synthesis Flow (Self-Correction TOML)

The actual process of evolution is governed by a declarative `Qianji` manifest.

### 4.1 Pipeline Nodes

1.  **Grand_Auditor**: Extracts failure DNA from `ZhenfaAuditSink` traces and outputs evidence-grounded root causes.
2.  **Soul_Forger**: Uses the Artisan Soul Blueprint to synthesize one targeted persona upgrade for the dominant gap.
3.  **Forge_Guard**: Performs formal audit and score gating to prevent degeneration before promotion.

### 4.2 Feedback Loop

Successfully forged souls are saved to the `autonomous-forged` skill path and immediately re-indexed, enabling the Agent to solve previously "impossible" tasks in the next reasoning cycle.
