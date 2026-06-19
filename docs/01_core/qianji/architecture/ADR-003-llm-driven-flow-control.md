---
type: knowledge
title: "ADR-003: LLM-Driven Flow Control (Synaptic Flow V2)"
status: "Draft"
date: "2026-02-26"
category: "architecture"
tags:
  - qianji
  - adr
  - synaptic-flow
  - adversarial-loop
saliency_base: 8.5
decay_rate: 0.01
metadata:
  title: "ADR-003: LLM-Driven Flow Control (Synaptic Flow V2)"
---

# ADR-003: LLM-Driven Flow Control (Synaptic Flow V2)

## 1. Context and Problem Statement

The "Adversarial Agenda Validation Loop" requires a proactive "Strict Teacher" node to critique draft schedules proposed by the "Agenda Steward". If the draft schedule demonstrates cognitive overload or ignores historical failures, the Teacher must force the Steward to retry via a `RetryNodes` control instruction.

Currently, the `Qianji` workflow engine has a severe architectural gap:

- The `formal_audit` mechanism is hardcoded logic (checking simple boolean properties like `has_grounding`). It cannot utilize LLMs.
- The `llm` mechanism is purely analytical. It executes prompts and stores the output in the context, but it **cannot emit routing instructions** (like `RetryNodes` or `SelectBranch`).

This prevents us from building truly autonomous, LLM-driven adversarial loops where an AI critic dictates the control flow of the workflow engine.

## 2. Decision

We will implement **Mechanism Fusion** in the `QianjiCompiler`. We will upgrade the `FormalAuditMechanism` to optionally operate as an **LLM-Driven Flow Controller**.

If a `formal_audit` node definition contains `[nodes.annotation]` and `[nodes.llm]` bindings, the compiler will construct an **LLM-Augmented Audit Mechanism**.

### The Execution Lifecycle of the LLM-Augmented Audit:

1. **Annotation Phase**: Run `ContextAnnotator` logic to prepare the prompt using the `Strict Teacher` persona and `critique_agenda.j2` template.
2. **Inference Phase**: Invoke the `LlmClient` to get the critique output.
3. **Parse Phase**: Parse the resulting `XML-Lite` string to extract the numerical `<score>`.
4. **Flow Decision Phase**:
   - If `score < 0.8` (or configurable threshold): Emit `FlowInstruction::RetryNodes` and append the critique to the context.
   - If `score >= 0.8`: Emit `FlowInstruction::Continue`.

## 3. Technical Design

### 3.1 Upgrading the `FormalAuditMechanism`

We will modify `packages/rust/crates/xiuxian-qianji/src/executors/formal_audit.rs`. We will introduce a new variant or internal logic to handle LLM execution.

```rust
// Conceptual Design

pub struct LlmAugmentedAuditMechanism {
    // Composition of existing mechanisms for DRY
    pub annotator: Arc<ContextAnnotator>,
    pub llm_client: Arc<dyn LlmClient>,
    pub model: String,

    // Audit specific params
    pub threshold_score: f32,
    pub retry_target_ids: Vec<String>,
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for LlmAugmentedAuditMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        // 1. Run Annotator to build prompt snapshot
        let annotation_output = self.annotator.execute(context).await?;
        let prompt = annotation_output.data.get(&self.annotator.output_key).unwrap().as_str().unwrap();

        // 2. Run LLM
        let response = self.llm_client.chat(ChatRequest { ... }).await?;

        // 3. Parse Score from XML (e.g., <score>0.5</score>)
        let score = extract_xml_score(&response).unwrap_or(0.0);

        // 4. Decide Flow
        let mut data = serde_json::Map::new();
        data.insert(self.output_key.clone(), json!(response));

        if score < self.threshold_score {
            Ok(QianjiOutput {
                data: Value::Object(data),
                instruction: FlowInstruction::RetryNodes(self.retry_target_ids.clone()),
            })
        } else {
            Ok(QianjiOutput {
                data: Value::Object(data),
                instruction: FlowInstruction::Continue,
            })
        }
    }
}
```

### 3.2 Modifying `QianjiCompiler`

In `packages/rust/crates/xiuxian-qianji/src/engine/compiler.rs`, the `build_formal_audit_mechanism` function will inspect the `NodeDefinition`. If `node_def.annotation` or `node_def.llm` is present, it will instantiate `LlmAugmentedAuditMechanism`. Otherwise, it falls back to the legacy logical `FormalAuditMechanism`.

## 4. Addressing the Data Gap (Historical Reality)

To make the "Strict Teacher" effective, it needs real data. Currently, `critique_agenda.j2` expects `{{ wendao_search_results }}`.

To bridge this, we must ensure that the `agenda_flow.toml` workflow explicitly includes a **Wendao Retrieval Node** _before_ the Strict Teacher node, using the direct Wendao search boundary (via an `action` or `knowledge` node in Qianji) to query the user's past carryover metrics and inject them into the context.

## 5. Implementation Plan

1. **Rust (Qianji)**: Implement `LlmAugmentedAuditMechanism` in `xiuxian-qianji` and update the `QianjiCompiler`.
2. **Rust (Regex/Parser)**: Implement a lightweight XML tag extractor in the audit mechanism to securely parse `<score>`.
3. **TOML Updates**: Update `agenda_flow.toml` to pass the correct parameters (like `threshold_score = 0.8`) and ensure the graph structure matches this new capability.
