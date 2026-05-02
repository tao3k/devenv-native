# Core Boundary

:PROPERTIES:
:ID: f92521584a38a3602c16ad2eeabbd32eb4f60eb9
:TYPE: CORE
:STATUS: DRAFT
:END:

Architecture boundary note for the `xiuxian-llm` library crate. Capture core responsibilities, integration edges, and invariants here.

The active project-policy gate uses `rust-lang-project-harness` without disabled rules. The public module tree exposes explicit embedding, LLM, model-runtime, and web boundaries; provider-specific Anthropic message handling is split into focused child modules for routing, media normalization, HTTP retry, request conversion, response parsing, and provider construction.
