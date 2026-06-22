# Phase 7.4: Global Broadcast Hub & Signal Fusion

## 1. Signal Registry

- **Type**: `tokio::sync::broadcast` (Multi-producer, Multi-consumer).
- **Location**: Injected into `ZhenfaContext` as a shared extension.

## 2. Heterogeneous Event Fusion

- **Mechanism**: The `ZhenfaPipeline` polls the `SignalRegistry` non-blockingly during `process_line`.
- **Injection**: External signals (e.g., `SemanticDrift` from Sentinel) are "plugged" into the primary agent stream as `ExternalSignal` payloads.

## 3. Neural Protection Layer (v3.5)

- **TokenBucket Rate Limiter**: Lock-free implementation (100 tokens/sec) prevents notification storms during massive source code refactoring.
- **Type-Safe Bridging**: Converts `ObservationSignal` (Wendao) to `ExternalSignal` (Zhenfa) at the bridge layer.
