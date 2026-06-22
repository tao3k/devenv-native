# Zhenfa Streaming Pipeline: The Sovereign Gateway

## 1. Unified Streaming Parser

- **Abstract Layer**: Collapses Claude (NDJSON), Gemini (SSE), and Codex protocols into a single `ZhenfaStreamingEvent`.
- **Thought Separation**: Extracts agent reasoning output for cognitive auditing before text reaches the UI.

## 2. Logic Gate (Incremental XSD)

- **Mechanism**: Validates XML fragments on the fly using a streaming state machine.
- **Interception**: Prevents "Logic Hallucinations" (e.g., non-sequential steps or invalid tags) by emitting `LogicGateError` before chunk finalization.

## 3. Cognitive Supervisor

- **Dimensions**: Classifies thoughts into Meta, Operational, Epistemic, or Instrumental.
- **Early-Halt**: Automatically triggers a `should_halt` signal if coherence scores drop below the configured threshold.

## 4. Project Policy Gate

- **Harness**: `rust-lang-project-harness` runs without disabled rules for source, unit, and integration project-policy gates.
- **Module Shape**: Streaming event contracts live in `src/transmuter/streaming/event.rs`, while `mod.rs` remains an interface and re-export boundary.
