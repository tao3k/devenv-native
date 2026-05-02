# xiuxian-zhixing (知行)

> **Naming origin**: "Xiuxian" (修仙) and "Zhixing-Heyi" (知行合一), used in this project to represent a knowledge-to-action execution loop.

**"Unity of Knowledge and Action" — The Sovereign Manifestation Engine.**

`xiuxian-zhixing` is the core execution and manifestation layer of the CyberXiuXian ecosystem. Inspired by the philosophy of _Zhixing-Heyi_ (知行合一), it bridges the gap between semantic intent and real-world outcomes, ensuring that "Knowing" (知) and "Doing" (行) are unified into a single synaptic loop.

---

## 🏛️ Architectural Mandate

In the Trinity of Agent OS, **Zhixing** serves as the "Physical Hand":

- **Wendao (Memory)**: Provides the _Knowledge_ (知).
- **Qianji (Workflow)**: Provides the _Logic_.
- **Zhixing (Action)**: Provides the _Action_ (行).

## 🧠 Core Features

### 1. Manifestation of Intent (显化)

Beyond simple task tracking, `xiuxian-zhixing` ensures that if an intent is accepted into the **Sovereign Agenda**, it becomes a physically indexable and executable artifact. Action is the ultimate validation of Knowledge.

### 2. Markdown-First Persistence

All states—agendas, journals, and reminders—are persisted as human-readable, machine-indexable Markdown artifacts. This ensures **Total Data Sovereignty** and seamless integration with the **Wendao VFS**.

### 3. Reflective Feedback Loops

Execution results are fed back into the **Wendao** knowledge graph as "Episodic Memories," allowing future "Knowledge" cycles to be refined by past "Action."

## 🛠️ Domain Layers

- **Agenda**: Management of high-priority research intents and timelines.
- **Journal**: Immutable record of synaptic execution and manifestation.
- **Reminders**: Time-aware semantic triggers for agentic re-activation.
- **Blockers**: Formal detection and handling of physical obstacles to the "Unity of Action."

## Project Policy Gate

`xiuxian-zhixing` uses `rust-lang-project-harness` as its active
project-policy gate with no disabled-rule baseline. The gate is mounted from
the library, root unit-test target, and shared lib-policy target.

The strict gate requires all diagnostics to be closed, including informational
agent-policy output. Current owner-boundary fixes connect the action compiler,
config, and interface modules to the crate tree, keep facade files thin, move
embedded resources to `src/resources.rs`, remove duplicate public names, and
replace test glob imports with explicit owner imports.

Current verification:

- `cargo test -p xiuxian-zhixing enforce_rust_project_harness_gate -- --nocapture`
- `cargo test -p xiuxian-zhixing --lib --test unit_test`
- `cargo test -p xiuxian-zhixing --all-features`
- `cargo fmt --package xiuxian-zhixing --check`
- `cargo clippy -p xiuxian-zhixing --all-targets --all-features -- -D warnings`

---

**CyberXiuXian Artisan Workshop** - _Unifying Knowledge and Action in the Neural Backbone._
