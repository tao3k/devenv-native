# xiuxian-qianhuan

Xiuxian-Qianhuan injection engine for Omni Rust runtime.

## Naming Origin

This injection system is branded as **Xiuxian-Qianhuan**:

- **Xiuxian**: continuous cultivation and refinement of reasoning quality.
- **Qianhuan**: thousand transformations, representing fast role adaptation and mixed-domain composition.
- Chinese name: **修仙-千幻** (Xiuxian-Qianhuan).
- Naming reference: _cultivation + thousand transformations_.

In runtime terms, Xiuxian-Qianhuan means:

- the system can transform into different role packs for different workflows,
- combine specialist perspectives in one turn,
- inject high-signal context into execution with deterministic policy and bounded cost.

## Why This Crate Exists

`xiuxian-qianhuan` is the dedicated context injection interface layer.
It is intentionally separate from planning and execution engines.

Responsibilities:

- Parse and normalize structured injection payloads.
- Enforce bounded injection windows and deterministic output.
- Serve as the foundation for role-mix and classified injection modes.
- Provide a stable contract for runtime context assembly.
- Expose a Rust-only orchestration and persona surface for native runtime consumers.

Non-responsibilities:

- No workflow planning (Graph responsibility).
- No tool execution loop (ReAct responsibility).
- No global policy arbitration (Omega responsibility).
- No Python or PyO3 binding surface.

## Runtime Position

The crate is embedded between policy and execution:

1. Omega selects policy, mode, and role mix.
2. Xiuxian-Qianhuan assembles injection snapshot.
3. Graph/ReAct consume the snapshot.
4. Reflection and memory evolution run after execution.

## Injection Model

Current foundation in this crate:

- XML Q&A payload parsing (`<system_prompt_injection><qa>...`).
- Bounded window with size limits (`InjectionWindowConfig`).
- Canonical rendering and normalization for deterministic replay.

Planned extension surface:

- `single` mode: one compact injection block.
- `classified` mode: category-aware injection with per-category budgets.
- `hybrid` mode: role-mix + classified blocks in one immutable snapshot.

## Key Types

- `InjectionWindowConfig`
- `QaEntry`
- `SystemPromptInjectionWindow`
- `InjectionError`
- `SYSTEM_PROMPT_INJECTION_TAG`
- `PromptContextBlock`
- `InjectionPolicy`
- `InjectionSnapshot`
- `RoleMixProfile`
- `ManifestationManager`
- `EmbeddedManifestationTemplateCatalog`

## Embedded Template Catalog

`xiuxian-qianhuan` now also exposes
`EmbeddedManifestationTemplateCatalog` as the thin lazy surface for
embedded-only markdown or control-plane renderers.

Use this when a consumer crate needs:

- bundled Jinja/Tera templates only,
- lazy shared bootstrap over `ManifestationManager::new_with_embedded_templates`,
- stable `render_text(...)` or `render_lines(...)` calls without owning a
  local `OnceLock<Result<ManifestationManager, _>>` wrapper.
- checked-in template files that can be wired through `include_str!` while the
  rendering bootstrap stays in `xiuxian-qianhuan`.

## Example

```rust
use xiuxian_qianhuan::{InjectionWindowConfig, SystemPromptInjectionWindow};

let raw = r#"
<system_prompt_injection>
  <qa>
    <q>What is the active runtime constraint?</q>
    <a>Keep execution deterministic and bounded.</a>
    <source>omega.policy</source>
  </qa>
</system_prompt_injection>
"#;

let normalized = SystemPromptInjectionWindow::normalize_xml(raw, InjectionWindowConfig::default())?;
println!("{normalized}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Architecture Boundaries

- Core memory policy/lifecycle belongs to Rust memory core (`omni-memory`), not this crate.
- This crate focuses on context injection assembly only.
- External clients can use memory/knowledge tool facades, while runtime policy stays in Rust core packages.

## Project Policy Gate

`xiuxian-qianhuan` uses `rust-lang-project-harness` as its active
project-policy gate with no disabled-rule baseline. The gate is mounted from
the library, root unit-test target, and shared lib-policy target.

The strict gate requires all diagnostics to be closed, including informational
agent-policy output. Current owner-boundary fixes keep `interface/mod.rs` as
an interface-only re-export module, move the public manifestation trait to
`interface/api.rs`, and add concise intent docs to public and branch modules.

Current verification:

- `cargo test -p xiuxian-qianhuan enforce_rust_project_harness_gate -- --nocapture`
- `cargo test -p xiuxian-qianhuan --lib --test unit_test`
- `cargo test -p xiuxian-qianhuan --all-features`
- `cargo fmt --package xiuxian-qianhuan --check`
- `cargo clippy -p xiuxian-qianhuan --all-targets --all-features -- -D warnings`

## Related Docs

- `docs/plans/omega-graph-react-rust-unification.md`
- `docs/plans/knowledge-injection-memory-evolution-architecture.md`
