# ADR-012: Protocol Hygiene & Tool Call Integrity Guard

- **Status**: Accepted
- **Date**: 2026-03-06
- **Context**: [RE-OPENED] Phase 18: The Synaptic Memory Matrix

## Context

The system integration with OpenAI-compatible `/responses` (Realtime/Legacy) and other strict LLM providers frequently encountered the error: `No tool call found for function call output with call_id ...`.

Investigation revealed that:

1.  **Orphaned Messages**: The `BoundedStore` (sliding window) would sometimes prune the original `Assistant` request but keep the `Tool` result, leading to a protocol violation.
2.  **Native Tool Disconnect**: Native tools triggered via intent recognition (instead of model output) often lacked a formal `assistant` request message in the history, which is required by LLM providers.
3.  **ID Mismatch**: Composite IDs (e.g., `call_id|fc_id`) from some gateways were not normalized, causing string mismatch errors.

## Decision

We have implemented a mandatory **`Protocol Hygiene`** layer in `xiuxian-daochang::llm::protocol::hygiene`.

1.  **Stateful Validation**: The `enforce_tool_message_integrity` function uses a `PendingAssistant` state machine to scan the message history before every LLM dispatch.
2.  **Chain Enforcement**: Any `assistant` message with `tool_calls` must be followed by matching `tool` messages. If the chain is incomplete or an ID is missing, the entire chain (assistant + partial tools) is purged to protect the provider session.
3.  **Automatic Normalization**: Tool IDs are normalized (splitting at `|` and trimming) to ensure they match the provider's expectations regardless of gateway metadata.
4.  **Reporting**: A `ToolMessageIntegrityReport` is generated for every request, providing visibility into dropped "orphan" messages for monitoring and diagnostics.

## Consequences

- **Stability**: Prevents 400 Errors from LLM providers due to malformed tool-call chains.
- **Protocol Compliance**: Ensures the system remains 100% compliant with OpenAI and Anthropic tool calling specifications.
- **Native Tool Support**: Provides a foundation for "Shadow Call Backfill," where the system can synthesize the necessary protocol messages for native actions.
- **Transparency**: Architectural visibility into "transcript health" is now a first-class citizen of the LLM client.

---

_True Integrity is Sovereign-Driven_
