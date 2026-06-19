---
id: "20260315154000"
type: knowledge
title: "Spec: Zhenfa Pipeline (The Cognitive Shield)"
category: "architecture"
tags:
  - zhenfa
  - pipeline
  - orchestration
  - streaming-defense
saliency_base: 9.5
decay_rate: 0.01
metadata:
  title: "Spec: Zhenfa Pipeline (The Cognitive Shield)"
---

# Spec: Zhenfa Pipeline (The Cognitive Shield)

## 1. Overview

The **ZhenfaPipeline** is the primary high-level interface for the **xiuxian-zhenfa** streaming kernel. It encapsulates the "Trinity" of defense mechanisms into a single, industrial-grade orchestration unit.

## 2. Orchestration Logic

The pipeline manages the state transition of a single agent stream:

1. **Transmutation**: Maps raw CLI chunks to `ZhenfaStreamingEvent` (Zero-copy).
2. **Validation**: Incremental XSD checking via the `LogicGate`.
3. **Supervision**: Reasoning coherence scoring via the `CognitiveSupervisor`.

## 3. Physical Interface

```rust
pub struct ZhenfaPipeline {
    parser: Box<dyn StreamingTransmuter>,
    logic_gate: LogicGate,
    supervisor: CognitiveSupervisor,
    // ... options
}

impl ZhenfaPipeline {
    pub fn process_line(&mut self, line: &str) -> Result<Vec<PipelineOutput>, PipelineError>;
    pub fn should_halt(&self) -> bool;
    pub fn finalize(&mut self) -> Result<Option<StreamingOutcome>, PipelineError>;
}
```

## 4. Implementation Highlights

- **Error Interrupts**: Immediately returns `PipelineError::ValidationError` if the XSD contract is breached.
- **Early-Halt**: Automatically flags `should_halt` if the coherence score drops below the threshold (default: 0.3).
- **Industrial Hardening**: Uses static XSD maps and `VecDeque` history for constant-time performance.

---

## Linked Notes

- Parent MOC: [[20260315151000-zhenfa-matrix-moc]]
- Core Component: [[20260315152000-unified-streaming-parser-spec]]
- Consumer: Qianji runtime nodes
