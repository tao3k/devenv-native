---
type: knowledge
title: "RFC: Data-Centric Workflow Orchestration on Wendao Relations"
category: "rfc"
status: "draft"
authors:
  - codex
created: 2026-03-26
tags:
  - rfc
  - qianji
  - wendao
  - datafusion
  - arrow
  - multi-agent
  - workflow
---

# RFC: Data-Centric Workflow Orchestration on Wendao Relations

## 1. Summary

This RFC proposes a paradigm shift for the **Qianji Workflow Engine**: evolving from a traditional task-based trigger system to a **Data-Centric Orchestration** model built on top of the Wendao Query Engine. Qianji will treat workflows as orchestration plans over Wendao-produced relations where data is passed between agents as zero-copy Arrow `RecordBatches`.

Qianji does **not** own query planning, retrieval planning, graph planning, or storage policy. Those concerns belong to Wendao. Qianji owns workflow orchestration, agent scheduling, audit and repair loops, and relation handoff between workflow stages.

## 2. Motivation

The current Qianji architecture relies on serializing agent outputs (often JSON) and passing them through the Zhenfa signal bus. While robust, this approach encounters significant bottlenecks when:

1. **Large Contexts**: Passing large AST structures or reference sets between agents incurs heavy CPU and memory overhead.
2. **Tight Loops**: Multi-agent loops (e.g., iterative document fixing) suffer from repeated serialization/deserialization cycles.
3. **Black-box Reasoning**: Users cannot see the "data funnel" as an agent filters millions of rows down to a few insights.

By embracing a Data-Centric model on top of Wendao relations, we can achieve **sub-100ms multi-agent handoffs** even with GB-scale datasets while keeping query semantics centralized in Wendao.

## 3. Core Architectural Changes

### 3.1 Zero-Copy Agent Handoff (Arrow Handoff)

Qianji will standardize on **Apache Arrow 58** as the inter-agent memory contract.

- **Mechanism**: Instead of piping JSON strings, Qianji passes `Arc<RecordBatch>` handles.
- **Benefit**: An "AST Extraction Agent" defined in Qianhuan can produce a batch of 50,000 nodes, and a "Diagnostic Agent" can consume it instantly without touching the heap.

### 3.2 Workflow-as-an-Orchestration-Plan

Workflows are no longer static. They become orchestration plans over Wendao relations and Wendao operator outputs.

- **Dynamic Partitioning**: Qianji consumes Wendao-produced relations that have already been pruned, partitioned, and materialized by the Wendao Query Engine, spawning "Agent Workers" only for relevant data shards.
- **Relation-Aware Scheduling**: Agent steps consume typed relations and emit typed relations. Qianji schedules these steps without claiming ownership of DataFusion planner internals.

### 3.3 Relational Skepticism (Hallucination Detection)

The **Skeptic (Auditor)** role is upgraded from a prompt-based check to a **Relational Join** check.

- **Consistency Joins**: Qianji validates LLM-generated suggestions by issuing audit queries against Wendao relations and performing contradiction checks against Wendao-produced truth tables.
- **Example**: If an agent suggests a function signature change, Qianji requests the relevant truth relation from Wendao and verifies whether that signature already exists before admitting the suggestion into the workflow.

## 4. Proposed Workflow Stage Model

Qianji should introduce workflow stages, not a competing query operator model.

1. **`agent_step(relation, prompt_template)`**: Applies an agent-defined transformation to rows or row groups in an Arrow relation.
2. **`audit_step(relation, audit_goal)`**: Verifies agent outputs against Wendao truth relations or explicit invariants.
3. **`reduce_step(relation, goal)`**: Summarizes a columnar result set into a structured insight card.
4. **`graph_context_step(seed_relation, context_goal)`**: Requests graph-adjacent context from Wendao and enriches the workflow state with the returned relation.

These workflow stages must remain above the Wendao query layer. Qianji may compose Wendao queries, but it should not redefine retrieval or graph operators that already belong to Wendao.

## 5. Integration with Zhenfa Stream Processing

Qianji will leverage the **Unified Streaming Parser** from Zhenfa:

- **Streaming Telemetry**: As Wendao executes query plans and agents consume the returned relations, Qianji will emit `ZhenfaStreamingEvents` showing the "Data Funnel" in the UI (e.g., "Scanning 1M lines... 500 potential matches... LLM refining...").
- **Thought Separation**: Intercepting the `Thinking Process` of agents during the handoff, allowing the **Cognitive Supervisor** to halt workflows if the reasoning logic diverges from the data schema.

## 5.1 Qianji-Wendao Orchestration Bridge

This section defines the bridge between Qianji workflow orchestration and the Wendao query engine.

### 5.1.1 Invocation boundary

Qianji should invoke Wendao through typed query requests or typed operator requests.

Qianji must not:

1. own Wendao planner internals
2. rewrite Wendao storage policy
3. re-implement retrieval or graph operators locally

Qianji may:

1. request a relation from Wendao
2. request graph context from Wendao
3. request truth relations for audit
4. consume explain and telemetry streams emitted by Wendao

### 5.1.2 Minimal bridge interface

The bridge should be able to express at least four operations:

1. `run_query(request) -> relation`
2. `run_operator(request) -> relation`
3. `explain(request) -> explain_stream`
4. `telemetry(request) -> telemetry_stream`

The exact Rust API may evolve, but the ownership rule should remain fixed: Qianji asks; Wendao resolves.

### 5.1.3 Workflow state relation

Qianji should maintain workflow state as Arrow-native relations rather than opaque JSON envelopes wherever practical.

This statement is about Qianji's logical state model, not storage ownership.
The runtime persistence for checkpoint, resume, and coordination state remains
Valkey-backed. DuckDB is not a checkpoint, resume, or coordination-state
store in this architecture. A bounded DuckDB data-store adapter may persist
workflow-local data records or stage materializations, but those records do
not replace Valkey checkpoint truth.

The minimal state relation should include fields such as:

1. `workflow_id`
2. `step_id`
3. `stage_kind`
4. `input_relation_id`
5. `output_relation_id`
6. `status`
7. `assigned_agent`
8. `audit_state`
9. `started_at`
10. `updated_at`

This relation is owned by Qianji. It is not a substitute for Wendao query outputs.

When a workflow stage later uses a bounded local relation engine, that engine
only computes over already materialized stage inputs or outputs. It does not
replace the workflow-state layer.

### 5.1.4 Explain and telemetry flow

The flow of observability should be:

1. Wendao emits explain-plan and execution telemetry for query and operator execution
2. Qianji binds those signals to workflow stages
3. Zhenfa renders the unified execution narrative

This preserves causal clarity:

1. Wendao explains why a relation exists
2. Qianji explains why a workflow step ran
3. Zhenfa renders both in a single user-facing stream

### 5.1.5 Bounded local relation engines remain downstream helpers

Stage-local relation analytics may later use a bounded in-process execution
helper over already materialized Arrow batches. The current bounded DuckDB
direction is tracked in
[RFC: DuckDB as a Bounded In-Process Analytic Lane for Wendao and Qianji](../../../../../../docs/rfcs/2026-04-08-wendao-qianji-duckdb-bounded-analytics-rfc.md).

That direction does not change the ownership split defined in this RFC:

1. Wendao still owns retrieval planning and storage policy
2. Qianji still owns workflow-stage orchestration and relation handoff
3. Valkey continues to own checkpoint persistence, resume state, and transient
   coordination
4. DuckDB, if later used, remains only a stage-local compute and data-plane
   helper for bounded workflow records, joins, rollups, contradiction checks,
   or reduce shaping over already materialized Arrow relations

## 6. Implementation Phases

### Phase 1: Arrow Interface Handoff

- Implement `WorkflowContext` that holds `BTreeMap<String, Arc<RecordBatch>>`.
- Update Qianhuan Agent definitions to accept and return Arrow batches.

### Phase 2: Wendao Relation Integration

- Integrate Qianji workflow stages with Wendao query outputs and explain streams.
- Implement the `Relational Skeptic` by requesting audit relations from Wendao rather than embedding planner ownership in Qianji.

### Phase 3: Streaming Visualization

- Bind Wendao execution telemetry and workflow-stage telemetry to the Qianji UI.
- Show real-time "Data Flow" animations in the Workflow editor.

## 7. Success Metrics

- **Handoff Latency**: Inter-agent data transfer for 10MB of records < 1ms.
- **Audit Accuracy**: 95%+ of AST-related hallucinations caught by relation-backed audit checks.
- **User Trust**: Clear visual correlation between "Raw Data" and "AI Conclusion" in the Studio.

---

_Document Date: 2026-03-26_
_Status: RFC-QIANJI-001_
