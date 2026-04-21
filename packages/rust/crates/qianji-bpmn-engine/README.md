# qianji-bpmn-engine

Bounded BPMN and DMN workflow engine ownership for Qianji.

## Responsibility

`qianji-bpmn-engine` owns the engine-side core for:

- BPMN parsing, bounded execution, and host-blocked workflow runtime
- DMN parsing, bounded evaluation, and LLM-friendly lint diagnostics
- checkpoint codecs plus distributed Valkey-backed checkpoint ownership
- lightweight local checkpoint persistence behind the `sqlite` feature
- stable diagnostic surfaces that power `qianji lint --bpmn` and
  `qianji lint --dmn`

## Structural Notes

- Medium or complex features should stay folder-first.
- `src/lint/dmn/` is the current DMN lint owner for entry dispatch,
  document guidance, contract guidance, snapshot helpers, decision helpers,
  evidence mapping, and unexpected-error fallback.
- `mod.rs` files are interface seams only and should not regrow hidden
  implementation buckets.

## Non-Goals

- This crate does not promise full BPMN or DMN parity yet.
- Adapter-specific orchestration belongs in higher layers such as
  `xiuxian-qianji`, not in the engine core.
- DMN widening should stay incremental and preserve LLM-friendly repair
  guidance rather than trading precision for broad but lossy support.
