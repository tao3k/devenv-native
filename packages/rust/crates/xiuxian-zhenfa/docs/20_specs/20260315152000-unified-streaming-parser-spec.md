---
id: "20260315152000"
type: knowledge
title: "Feature: Unified Streaming Parser (Transmuter)"
category: "features"
tags:
  - zhenfa
  - streaming
  - zero-copy
  - high-performance
  - industrial-grade
saliency_base: 9.0
decay_rate: 0.01
metadata:
  title: "Feature: Unified Streaming Parser (Transmuter)"
---

# Feature: Unified Streaming Parser (Transmuter)

## 1. Overview & Goal

The **Unified Streaming Parser** is a high-performance cognitive gateway designed to parse, validate, and display real-time output from multiple agent CLIs (Claude Code, Gemini CLI, Codex).

**Primary Goal**: To achieve **Industrial-Grade Reliability** through zero-copy memory patterns and real-time cognitive monitoring with sub-10ms latency.

## 2. Key Features

### 2.1. Symmetrical Zero-Copy Protocol (V1.5)

Every text delta and thought fragment is wrapped in `Arc<str>`.

- **Physical Symmetry**: Aligns all layers (Parsing, Logic Gate, Supervisor) around the `Arc<str>` model.
- **Zero Allocation**: Eliminates heap allocations during high-frequency token ingestion.

### 2.2. Industrial Performance Hardening (V3.1)

The implementation optimizes hot-path execution and initialization:

- **Static Constraint Map**: Uses `once_cell::sync::Lazy` for zero-initialization overhead of XSD schemas.
- **O(1) History Buffering**: Employs `VecDeque` for bounded cognitive history tracking, ensuring constant-time performance.
- **Fast Boundary Scanning**: Optimized cursor logic for real-time XML tag detection.

### 2.3. Incremental Logic Gate (Phase 1)

Performs "Hot Validation" on partial XML fragments as they stream in.

- **Early-Halt**: Immediately blocks agents violating the `qianji_plan.xsd` schema.
- **Step Linearity**: Enforces strict sequential processing of implementation steps.

### 2.4. Cognitive Supervisor (Phase 2)

Real-time "Process Supervision" using a three-dimensional cognitive model:

- **Dimensions**: Meta, Operational, and Epistemic categorization.
- **Coherence Scoring**: Heuristic-based hallucination detection with early-halt capabilities.

## 3. High-Performance Targets

| Metric                  | Achievement | Method                                     |
| :---------------------- | :---------- | :----------------------------------------- |
| **End-to-End Latency**  | < 10ms      | Zero-copy + Non-blocking Event Model       |
| **Initialization Cost** | 0 ns (Lazy) | Global static constraint singletons        |
| **Memory Overhead**     | Constant    | Sliding window history + Reference sharing |

## 4. Architecture (The Include Pattern)

The implementation is split into modular components for hyper-extensibility:

- `mod.rs`: Orchestrator & Provider Detection.
- `logic_gate.rs`: The industrial XSD validator (Static optimized).
- `supervisor.rs`: The cognitive scoring engine (O(1) window).
- `claude.rs` / `gemini.rs` / `codex.rs`: Provider-specific zero-copy parsers.

---

## Linked Notes

- Parent MOC: [[20260315151000-zhenfa-matrix-moc]]
- Contract System: [[20260315150000-zhenfa-contract-system-spec]]
- Design Blueprint: [[docs/data/blueprints/unified_streaming_parser]]
- Theoretical Foundations: [[20260315153000-theoretical-foundations-zhenfa]]
