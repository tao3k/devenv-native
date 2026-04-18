# Triple-A Addressing Protocol

:PROPERTIES:
:ID: wendao-triple-a
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: architecture, core, addressing
:STATUS: HARDENED
:END:

## Definition

The core resolution engine of Wendao, decoupling logical identity from physical line coordinates using AST-derived pointers.

## Protocol Layers

1. **Anchor**: Explicit identification via `:ID:` property drawers.
2. **AST Path**: Logical navigation via tree hierarchy (e.g., `/Arch/Storage`).
3. **Alias**: Content-based self-healing via Blake3 fingerprints.

:RELATIONS:
:LINKS: [[03_features/202_block_addressing|descriptive label]], [[01_core/102_atomic_mutation|descriptive label]], [[03_features/203_agentic_navigation|Agentic Navigation (wendao.agentic_nav)]]
:END:

---

:FOOTER:
:AUDITOR: auditor_neuron
:END:
