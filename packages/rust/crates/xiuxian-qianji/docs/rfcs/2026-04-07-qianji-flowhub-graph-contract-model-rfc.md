---
type: knowledge
title: "RFC: Qianji Flowhub Graph Contract Model"
category: "rfc"
status: "draft"
authors:
  - codex
created: 2026-04-07
tags:
  - rfc
  - qianji
  - flowhub
  - planning
  - manifest
  - validation
  - cli
---

# RFC: Qianji Flowhub Graph Contract Model

This RFC defines the target graph contract for the Flowhub-driven Codex
planning model. It describes the intended architecture and control surfaces;
it does not claim that every field or CLI subcommand is already implemented in
the current workspace.

The current code-backed coverage for this RFC cluster is tracked in
[Audit: Qianji RFC Implementation Coverage](2026-04-08-qianji-rfc-implementation-coverage-audit.md).

## 1. Terminology

### Plan Flow

A reusable flow in `qianji-flowhub` that defines how a normalized plan work
surface is generated for Codex.

### Plan-Track

The derived tracking subgraph of `plan`. It is used to validate boundary,
drift, status, and execution legality while Codex is working.

### Codex Operational Workdir

A bounded operational tracking surface materialized from the `plan` flow for
one active bounded slice.

This is a tracking surface, not a canonical documentation surface. This
terminology aligns with repository governance that treats ExecPlans and
similar artifacts as hidden tracking records rather than stable canonical
docs.

### Flowhub

A reusable flow library. It stores reusable flows and subgraphs, not active
workdirs.

### Blueprint

The durable architectural contract governing a work lane. This remains
conceptually aligned with the blueprint role already defined in `AGENTS.md`.

## 2. Directory and Repository Semantics

This RFC distinguishes three things clearly.

### 2.1 Flowhub

`qianji-flowhub` is a reusable flow library.

It stores:

1. upstream constraint and design flows
2. `plan`
3. derived or related tracking subgraphs such as `plan-track`

It does not store active Codex work surfaces.

### 2.2 Plan Work Surface

The active Codex working surface is the materialized result of the `plan`
flow.

This RFC describes it conceptually as a materialized plan work surface rather
than treating a hidden cache path as the canonical architectural object.

### 2.3 Tracking Surfaces

ExecPlans, Codex operational workdirs, and similar execution records are
tracking surfaces. They may be referenced operationally, but RFC text should
not normalize hidden workspace paths as canonical documentation targets.

## 3. Core Model

The correct model is graph-first, not directory-first.

For the current demo lane, the intended semantic graph is:

```mermaid
flowchart LR
  coding --> rust
  rust --> style
  rust --> engineering_requirement["engineering requirement"]
  rust --> policy
  rust --> blueprint
  blueprint --> plan
  plan --> plan_track["plan-track"]
```

Interpretation:

1. `coding` is the upstream domain node
2. `rust` is the language specialization node
3. `style`, `engineering requirement`, and `policy` are constraint subgraphs
   under `rust`
4. `blueprint` is the design-contract layer derived from those constraints
5. `plan` is the execution-surface generation flow
6. `plan-track` is the derived guard and tracking subgraph for active Codex
   work

This graph is the semantic source shape. Directory layout exists only to
support this graph.

For the current live demo library, that support is intentionally minimal: the
live `qianji-flowhub/` root exposes only top-level node anchors
`coding/`, `rust/`, `blueprint/`, and `plan/`. Finer subnodes such as
`style`, `engineering requirement`, and `policy` remain semantic graph nodes
in the target model, but they are not represented as live nested Flowhub
directories unless a parent node explicitly declares them through
`[contract]`.

Live nodes may also own immediate Mermaid scenario-case graphs. For example,
the current `plan` node owns `codex-plan.mmd`, and that file is part of the
node contract rather than an ad hoc extra artifact. Mermaid parsing for those
scenario-case files should prefer a mature parser dependency; the current
implementation uses `merman-core`. The parsed graph identity should come from
module-owned `[[graph]].name` when declared, otherwise from the owning
Mermaid filename stem as `merimind_graph_name`, and never from the Mermaid
direction token such as `LR`. A first-version valid scenario-case graph must
cover every registered Flowhub module node required by the current root
contract, must include at least one edge between Flowhub module nodes, must
preserve one connected module backbone over those module nodes, and must not
introduce undeclared graph-node labels outside the recognized guard or
process vocabulary.

## 4. What `plan` Actually Is

This RFC does not treat `plan` as a generic content module.

`plan` is a generation and normalization flow.

Its responsibilities are:

1. define the canonical structure of a bounded plan work surface
2. define the initial plan-facing files that Codex should consume
3. define the initial graph view for Codex
4. define the validation entry surface for subsequent bounded execution

`plan` therefore produces an operational work surface, not merely a document
template.

The minimal bounded plan work surface consists of:

1. `qianji.toml`
2. `flowchart.mmd`
3. `blueprint/`
4. `plan/`

Task structure belongs inside the plan surface itself. A separate `tasks.md`
is not part of the normative surface.

## 5. What `plan-track` Actually Is

`plan-track` is not a separate business flow.

`plan-track` is the derived tracking subgraph of `plan`.

Its responsibilities are:

1. boundary checks
2. drift checks
3. status checks
4. execution legality checks

`plan-track` exists to guard active Codex execution inside the bounded plan
work surface.

It is a checking surface, not a separate authoring surface. In the normative
CLI model, `plan-track` checks are invoked through unified `qianji check`.

This matches the role that ExecPlan slices already play in repository
governance: they are bounded execution records with an explicit relationship
to an upstream blueprint.

## 6. CLI-First Control Surface

The normative interface is the `qianji` CLI.

Codex should not be expected to understand repository topology directly.
Codex should rely on the `qianji` CLI to:

1. obtain the plan flow
2. materialize the normalized plan work surface
3. show the active plan graph and primary content surfaces
4. run unified validation and `plan-track` checks
5. receive precise validation diagnostics

This is a better fit than making MCP the primary control plane, because the
CLI gives a more deterministic, reproducible, bounded interaction surface.

The existing `qianji` binary already demonstrates manifest-driven and
graph-oriented CLI behavior, including graph export and manifest execution
entrypoints.

The intended command surfaces are:

```bash
qianji show --graph <scenario.mmd>
qianji show --dir <plan-workdir>
qianji check --dir <plan-workdir>
```

## 6. Three-Layer Execution Model v0

The current implementation is intentionally split into three layers.

### 6.1 Execution Layer

The execution layer is Codex.

Codex is responsible for:

1. reading the graph contract
2. writing artifacts inside the bounded work surface
3. iterating on those artifacts in response to diagnostics

Codex does not define the constraint model. It executes inside the constraint
model.

### 6.2 Constraint Layer

The constraint layer is `qianji-flowhub`.

`qianji-flowhub` is responsible for:

1. providing the normative graph
2. providing upstream flow and plan or blueprint constraints
3. providing scenario contracts
4. providing the template boundary and backbone graph

The constraint layer is not the runtime surface and is not the localized
workdir. It is the contract source.

### 6.3 Evaluation Layer

The evaluation layer is `qianji check`.

`qianji check` is responsible for:

1. reading the localized contract from the current bounded work surface
2. evaluating the current artifact state against that localized contract
3. returning a continue or block outcome
4. emitting precise diagnostics

### 6.4 Localized Contract Principle

The active workdir manifest is the localized contract surface derived from
`qianji-flowhub`.

The localized contract is authored in two stages:

1. `qianji-flowhub` provides the source graph and constraint contract
2. Codex materializes the localized contract into `<plan-workdir>/qianji.toml`

Therefore, `qianji check` does not evaluate raw Flowhub state directly.
`qianji check` evaluates the bounded artifact state that Codex has already
materialized under the localized contract.

This principle is the most important execution rule for the current model:

> Qianji Flow constraints are authored in flowhub, localized into the bounded
> workdir by Codex, and evaluated by `qianji check` against the resulting
> artifact state.

`qianji show --graph` exposes one Flowhub graph companion directly from the
Flowhub library. Its responsibility is limited to the graph contract surface:
graph metadata together with the `Mermaid`, `Nodes`, `Module contract`, and
`Owning qianji.toml` sections. `qianji show --dir` exposes the current graph
entry surface and the main bounded content surfaces. `qianji check` unifies
`[[validation]]`, flowchart alignment, boundary checks, drift checks, and
status legality checks.

### 6.5 `qianji show --graph` Output Contract v0

`qianji show --graph <scenario.mmd>` is a graph-contract display surface only.

It must not:

1. run `check`
2. scan arbitrary repository directories
3. perform exact retrieval
4. run toolchain validator evaluation

Its responsibility is to provide enough graph-contract information for the
executor to inspect the declared Flowhub graph contract without inventing
extra Rust-owned business semantics.

The output contract is graph metadata plus four fixed sections.

#### Section 1: Graph Metadata

The surface must begin with:

1. graph name
2. graph path
3. graph topology
4. declared graph topology when present

For example:

```md
# Graph

Name: codex-plan
Path: ./qianji-flowhub/plan/codex-plan.mmd
Topology: bounded_loop
Declared topology: bounded_loop
```

#### Section 2: Raw Mermaid

The surface must include the original Mermaid graph verbatim in one fenced
`mermaid` block.

#### Section 3: Nodes

Each node must be rendered with exactly four slots:

1. `Kind`
2. `Role`
3. `Agent action`
4. `Next`

This keeps the graph contract executor-agnostic and bounded to first-order
materialization guidance.

#### Section 4: Module Contract

The surface must explicitly show the required module-owned contract entries
anchored by the owning Flowhub module.

#### Section 5: Owning `qianji.toml`

The surface must include the verbatim owning module manifest so the executor
can inspect the exact local Flowhub contract that declared the graph.

### 6.6 `qianji show --graph` Node Semantics Contract

The `Nodes` section is contract-owned rather than Rust-taxonomy-owned.

Each declared `[[graph.node]]` entry provides:

1. `label`
2. `kind`
3. `role`
4. `agent_action`

`kind` is an opaque contract-owned string. Rust parses, validates, and renders
it, but Rust does not define an exhaustive business taxonomy for node kinds.
Live Flowhub modules may therefore use different kind spellings when the
contract owner needs them, such as `context`, `artifact`, `process`, or
`capability_contract`.

Node labels still use a minimal normalization rule for contract matching:

1. preserve the visible label text
2. strip Mermaid angle-bracket metadata segments during matching
3. do not infer taxonomy from the normalized value

When a node has no matching `[[graph.node]]` contract, the rendered node
semantics must stay defensive:

1. undeclared non-module nodes are outside the declared Flowhub graph contract
   vocabulary
2. registered module nodes without a matching `[[graph.node]]` entry are
   structural alignment drift rather than a source of inferred business meaning
3. the agent action must instruct the executor not to rely on that node until
   the Flowhub graph contract is corrected

### 6.7 `qianji show --graph` Next Edge Semantics v0

The `Next` slot is a flattened adjacency surface for outgoing edges.

It is intentionally minimal in v0:

1. it lists downstream node labels only
2. it preserves stable outgoing-edge order
3. it does not render a separate edge-kind column

The current edge semantics are:

1. backbone edge
2. fail edge
3. repair-loop edge

#### Backbone Edge

A backbone edge is the normal forward edge in the graph contract.

Examples:

1. `coding -> rust`
2. `rust -> blueprint`
3. `blueprint -> plan`

For backbone edges, `Next` means the ordinary downstream graph-contract step.

#### Fail Edge

A fail edge is a failure branch such as:

```text
surface check -- fail --> diagnostics
```

For fail edges, `Next` still lists the downstream label in the same flattened
surface, but the semantics are failure-oriented rather than forward-progress
oriented.

In v0, fail edges are not rendered in a separate slot. They remain visible
through:

1. the raw Mermaid section
2. the additional failure-oriented downstream label in `Next`

#### Repair-Loop Edge

A repair-loop edge is an edge that returns execution from a repair-oriented
process node back to the bounded writing path.

Example:

```text
diagnostics -> Codex write bounded surface
```

For repair-loop edges, `Next` means the bounded retry target after diagnostics
or repair guidance has been consumed.

#### Rendering Rule

The `Next` slot must therefore be read as:

1. ordinary downstream graph progression for backbone edges
2. failure branch targets for fail edges
3. bounded retry targets for repair-loop edges

This is intentionally sufficient for v0 because the raw Mermaid block remains
present above the node-semantics surface.

### 6.8 `qianji show --graph` Role and Agent Action Wording Contract v0

The `Role` slot and the `Agent action` slot have different wording rules.

They must not collapse into one mixed sentence type.

#### Role Wording

`Role` must stay descriptive.

It describes what the node means in the graph contract.

The preferred v0 shape is:

1. short
2. descriptive
3. noun-phrase or declarative-phrase oriented
4. free of imperative wording

Good examples:

1. `define the bounded blueprint surface`
2. `require the bounded work surface to exist`
3. `allow completion only when required guards and validators pass`

Bad examples:

1. `create and populate blueprint/`
2. `do not write outside the bounded work surface`
3. `repair artifacts and re-run qianji check`

Those are action instructions and therefore belong in `Agent action`, not in
`Role`.

#### Agent Action Wording

`Agent action` must stay imperative.

It tells the executor what to do next under the graph contract.

The preferred v0 shape is:

1. short
2. imperative
3. artifact-oriented
4. bounded to the current work surface

Good examples:

1. `create and populate blueprint/`
2. `ensure qianji.toml, flowchart.mmd, blueprint/, and plan/ exist`
3. `do not treat the slice as complete before qianji check passes`

Bad examples:

1. `define the bounded blueprint surface`
2. `require the bounded work surface to exist`
3. `allow completion only when required guards and validators pass`

Those are contract descriptions and therefore belong in `Role`, not in
`Agent action`.

#### Boundary Rule

The boundary between the two slots is:

1. `Role` answers: what does this node mean in the contract
2. `Agent action` answers: what should the executor do because of this node

This boundary must remain stable so the `Nodes` section stays easy to consume
for both humans and agent executors.

### 6.9 `unknown` Node Failure Semantics v0

`unknown` is not a soft taxonomy bucket. It is a contract-warning state.

It means that the current Mermaid graph contains at least one node label that
is:

1. outside the registered Flowhub module set
2. outside the allowed graph-node vocabulary for v0

The failure semantics differ by surface.

#### `show --graph`

In `qianji show --graph`, an `unknown` node must remain visible.

The graph-contract surface must not hide it and must not silently coerce it
into another known node kind.

Instead, the `Nodes` section must render:

1. `Kind: unknown`
2. a defensive `Role`
3. a defensive `Agent action`

This keeps the graph-contract surface honest for both humans and agent
executors.

#### `qianji check`

In `qianji check`, an `unknown` node must be treated as blocking graph drift.

It is not a warning-only state. It must prevent the graph from being accepted
as valid until the unknown node is removed, renamed, or formally adopted into
the contract vocabulary.

#### Materialize and Localized Workdir Use

`unknown` belongs to the graph-contract side, not to the localized workdir
contract side.

Therefore:

1. `qianji show --graph` may still display the graph contract with `unknown`
   nodes
2. the executor must not rely on those nodes when materializing the localized
   workdir contract
3. localized contract generation should remain bounded to the known v0
   contract surface

#### Rule

The v0 rule is:

1. visible in `show --graph`
2. blocking in `qianji check`
3. excluded from localized contract materialization guidance

This keeps the display surface informative, the evaluation surface strict, and
the localized workdir contract bounded to the currently accepted graph
vocabulary.

### 6.10 Module and Export Alignment v0

`qianji show --graph` is not only a free-form Mermaid preview. It also aligns
module nodes back to the live Flowhub contract.

#### Module Alignment

For v0, the Flowhub root `contract.register` list is the authoritative module
allowlist for `show --graph`.

A Mermaid node is treated as a Flowhub module node only when its visible label
matches one registered module ref from the Flowhub root contract.

Therefore:

1. registered module labels are the authoritative module vocabulary
2. a non-registered label must not be silently treated as a module node
3. labels outside the registered module set stay on the graph-node side and
   may become `unknown`

This keeps module alignment anchored to the live Flowhub root contract rather
than to ad hoc label inference.

#### Export Alignment

For every aligned module node, v0 resolves module exports at exactly two
surfaces:

1. `entry`
2. `ready`

These come from the module-local `qianji.toml` export contract.

This means the graph lane aligns module nodes to the same `entry -> ready`
surface already used elsewhere in Flowhub module contracts.

#### Display Rule

The primary five-section `show --graph` markdown contract remains focused on
graph-contract guidance. Therefore, v0 does not require a separate exported
surface list inside the main `Nodes` rendering block.

Instead:

1. module and export alignment must occur in the graph-show resolution path
2. export alignment is bounded to `entry` and `ready`
3. the primary markdown surface may stay focused on graph metadata, Mermaid,
   node semantics, expected work surface, and localized contract template

This keeps the user-facing graph surface compact while preserving
contract-backed module alignment underneath it.

### 6.11 Graph Path and Naming Contract v0

`qianji show --graph` must expose a stable graph-identity surface in the
metadata block.

For v0, that surface is:

1. `Name`
2. `Path`
3. `Kind`

The order must remain stable so both humans and agent executors can read the
graph header deterministically.

#### `Name`

For v0, `Name` must render the stable `merimind_graph_name`.

`merimind_graph_name` must resolve from module-owned `[[graph]].name` when
declared, otherwise from the owning Mermaid filename stem.

It must not derive from:

1. the Mermaid direction token such as `LR` or `TB`
2. the first node label
3. any in-graph title-like label or comment

Therefore:

1. `qianji-flowhub/plan/codex-plan.mmd` renders `Name: codex-plan`
2. `qianji-flowhub/wendao/docs-search.mmd` may render `Name: DOC_SEARCH`
   through `[[graph]].name = "DOC_SEARCH"`
3. changing the Mermaid direction token does not change graph identity
4. changing internal node labels does not change graph identity

This keeps graph identity anchored to the contracted scenario-case file rather
than to mutable graph body details while still allowing explicit,
module-owned names when the filename is only a transport path.

#### `Path`

For v0, `Path` must identify the owning Mermaid file.

It does not identify the future localized workdir and does not identify the
localized `flowchart.mmd` companion.

When the graph file is inside the active project root, `Path` should render as
a repo-root-relative path prefixed with `./`.

Therefore:

1. `qianji-flowhub/plan/codex-plan.mmd` renders as
   `Path: ./qianji-flowhub/plan/codex-plan.mmd`
2. the graph metadata surface stays portable across machines within the same
   repository checkout
3. the display surface stays distinct from localized workdir paths such as
   `<plan-workdir>/flowchart.mmd`

If the provided Mermaid file is outside the active project root, v0 may render
the caller-provided filesystem path instead.

#### Rule

The v0 naming and path rule is:

1. `Name` comes from `[[graph]].name` or the owning Mermaid filename-stem
   fallback as `merimind_graph_name`
2. `Path` points to the owning Mermaid file, not to the localized workdir
3. repo-owned graph files should render with repo-root-relative paths

### 6.12 Mermaid Graph Consumption Boundary v0

`qianji show --graph` has two Mermaid responsibilities:

1. render the raw Mermaid source verbatim
2. derive a bounded graph-contract semantic surface from that source

Those two responsibilities must not be conflated.

#### Raw Mermaid Surface

The `## Mermaid` section is the source-preservation surface.

Its job is to show the original graph text exactly as authored so humans and
agent executors can inspect the full scenario-case graph directly.

This surface may therefore contain Mermaid constructs that are richer than the
bounded v0 graph-contract semantics.

#### Consumed Semantic Surface

For v0, the graph-contract semantic surface is limited to first-order graph
structure:

1. the owning Mermaid file identity
2. normalized visible node labels
3. directed node adjacency

This bounded semantic surface is what feeds:

1. node taxonomy
2. `Next` adjacency rendering
3. module alignment
4. localized contract guidance

#### Out of Scope for v0 Semantics

For v0, Mermaid presentation features are not first-order graph-contract
inputs.

That includes:

1. direction tokens such as `LR`, `TB`, `RL`, and `BT`
2. decorative styling directives such as `style`, `class`, and `classDef`
3. interactive directives such as `click`
4. other presentation-oriented Mermaid syntax that does not introduce or
   rename the first-order node and edge structure

Those constructs may still appear in the raw Mermaid block, but they do not
change:

1. `merimind_graph_name`
2. node-kind classification
3. the primary `Next` surface
4. localized workdir contract guidance

#### Rule

The v0 Mermaid-boundary rule is:

1. preserve the full Mermaid source in `## Mermaid`
2. consume only first-order node and edge structure for graph-contract
   semantics
3. do not let Mermaid presentation directives expand the localized contract
   surface

This keeps `show --graph` honest to the authored graph while keeping the
executor-facing contract bounded and stable.

The active workdir manifest is the localized contract surface derived from
`qianji-flowhub`. `qianji check` evaluates the current artifact state against
this localized contract, not against the raw flow library directly.

The bounded plan work surface should already expose its primary semantic split
through filenames and top-level directories.

The bounded work surface for v0 is:

```text
<plan-workdir>/
  qianji.toml
  flowchart.mmd
  blueprint/
  plan/
```

In this surface:

1. `qianji.toml` is the localized contract
2. `flowchart.mmd` is the graph companion for the current slice
3. `blueprint/` is the bounded blueprint surface
4. `plan/` is the bounded execution surface

The localized workdir contract is intentionally small and already aligns with
the current `WorkdirManifest` structure:

```toml
version = 1

[plan]
name = "<plan-name>"
surface = ["flowchart.mmd", "blueprint", "plan"]

[check]
require = ["qianji.toml", "flowchart.mmd", "blueprint", "plan", "blueprint/**/*.md", "plan/**/*.md"]
flowchart = ["blueprint", "plan"]
```

This contract expresses:

1. the stable bounded-slice name
2. the top-level surfaces that `qianji show --dir` should expose
3. the minimum required surfaces and markdown artifacts
4. the principal surfaces that `flowchart.mmd` must stay aligned with

`tree` is not the context retrieval mechanism itself. It is only an optional
structural probe that helps Codex decide whether deeper inspection is
necessary.

Exact retrieval is not implemented through a Qianji-specific query language.
Qianji owns graph contract display, structural validation, and derived
tracking checks. Wendao owns exact knowledge retrieval.

Because the repository already includes DataFusion, SQL is the natural
retrieval surface. No additional retrieval DSL is introduced in
`qianji.toml`.

Therefore, the standard bounded execution sequence is:

1. `qianji show --graph <scenario.mmd>`
2. Codex materializes `<plan-workdir>/qianji.toml`, `flowchart.mmd`,
   `blueprint/`, and `plan/`
3. `qianji show --dir <plan-workdir>`
4. `qianji check --dir <plan-workdir>`

Deeper context is then obtained in four distinct stages:

1. graph-contract inspection through `qianji show --graph <scenario.mmd>`
2. structural inspection through filenames, directories, and optional `tree`
3. exact retrieval through Wendao SQL when deeper inspection is needed
4. validation through unified `qianji check`

Conceptual SQL examples:

```bash
wendao sql "select path, skeleton from markdown where path like 'blueprint/%'"
wendao sql "select path, skeleton from markdown where path like 'plan/%'"
```

## 7. Codex Working Loop

The normative loop begins only after the plan work surface has been
materialized.

### 7.1 Stage 1: Materialize

```text
Qianji CLI
-> load the plan flow from qianji-flowhub
-> materialize the normalized plan work surface
-> attach or derive the plan-track subgraph
-> run initial validation
```

### 7.2 Stage 2: Bounded Codex Execution

```text
1. qianji show --dir <plan-workdir>
2. Codex reads filename and directory shape
3. if needed, Codex uses tree only to decide whether deeper inspection is necessary
4. Codex uses Wendao SQL for exact retrieval over blueprint/ and plan/
5. Codex edits only files inside <plan-workdir>
6. qianji check --dir <plan-workdir>
7. Codex reads precise diagnostics and continues
```

Rule:

Codex must treat the active plan work surface as its bounded execution slice.
It must not treat the whole repository as its primary working context.

This is directly aligned with the bounded-slice model described for ExecPlans
in `AGENTS.md`.

## 8. Flowchart Placement

Each materialized plan work surface must contain a graph companion file at the
work-surface root:

```text
flowchart.mmd
```

This file is required because it is the most direct graph entry surface for
both humans and LLMs.

The roles are:

1. `qianji.toml`: formal contract source
2. `flowchart.mmd`: direct graph companion view
3. `blueprint/`: modular blueprint-local surface
4. `plan/`: modular plan-local surface

The `qianji` CLI must be able to validate that the flowchart remains aligned
with the active plan graph shape.

## 9. RFC Drift Corrections

The following older formulations are explicitly rejected.

Rejected:

1. treating a generic scenario source tree as the primary object
2. treating `plan` as a plain content module
3. treating `plan-track` as an unrelated parallel flow
4. treating full-repository traversal as normal Codex behavior
5. normalizing hidden cache paths as canonical documentation references
6. introducing a separate normative `tasks.md` beside the plan surface

Accepted:

1. `plan` as the bounded work-surface generation flow
2. `plan-track` as the derived tracking subgraph of `plan`
3. active Codex work as a bounded operational tracking surface
4. the CLI as the normative control plane
5. Wendao SQL as the exact retrieval surface
6. `tree` as an optional structural probe rather than the retrieval mechanism
7. `qianji.toml` as the structural and validation contract rather than a
   retrieval DSL surface

## 10. Scenario as Guard Graph

A scenario in Qianji Flow must be treated as a guard graph, not merely as a
content bundle.

The scenario is the graph contract.

The bounded plan work surface is the artifact state evaluated by that
contract.

Codex is the execution layer. Qianji Flow is the contract layer.

A scenario graph does not execute code directly. Instead, it defines:

1. what Codex may write
2. what surfaces must exist
3. what artifacts must align
4. what guards must pass
5. when the current bounded slice may advance

A bounded slice must not be accepted as complete unless the required done gate
passes.

This section defines the target model. It does not claim that every guard
category below is already implemented in the current CLI.

### 10.1 Node Classes

A first-version target model should recognize four node classes.

#### Context Nodes

Context nodes define upstream semantic constraints.

Examples:

1. `coding`
2. `rust`
3. `style`
4. `engineering_requirement`
5. `policy`

These nodes constrain downstream artifact surfaces but do not directly
validate files by themselves.

In the current live Flowhub demo tree, some context nodes may exist only as
semantic nodes in `flowchart.mmd` or scenario composition, not as checked-in
child directories.

#### Artifact Nodes

Artifact nodes correspond to concrete bounded work-surface artifacts.

Examples:

1. `flowchart`
2. `blueprint_surface`
3. `plan_surface`

These nodes must map to actual files or directories inside the bounded work
surface.

#### Guard Nodes

Guard nodes are programmatic checks over artifact state.

Examples:

1. `surface_exists`
2. `flowchart_alignment`
3. `boundary_check`
4. `drift_check`
5. `status_legality`

These nodes should be modeled as graph concerns rather than only as ad hoc
runtime hooks.

#### Done Gate Nodes

A `done_gate` node decides whether the current bounded slice may be accepted
as complete.

A `done_gate` must not pass until all required upstream guard nodes for that
lane have passed.

Task completion must therefore be guard-backed, not self-declared by the
model.

### 10.2 Bounded Evaluation Surface

The scenario guard graph must be evaluated against a bounded work surface with
the following minimum shape:

```text
<plan-workdir>/
  qianji.toml
  flowchart.mmd
  blueprint/
  plan/
```

In this surface:

1. `qianji.toml` is the local contract
2. `flowchart.mmd` is the direct graph companion
3. `blueprint/` and `plan/` are the bounded artifact surfaces

This RFC must continue to describe the work surface conceptually as
`<plan-workdir>`. It must not normalize hidden tracking paths as canonical
architecture subjects.

### 10.3 CLI Consequence

Under this model:

1. `qianji show` must remain the graph and contract display surface
2. `qianji check` must remain the unified guard-graph evaluation surface

`qianji show` must expose:

1. the graph companion
2. the visible bounded surfaces
3. the first-order structure Codex should consume

`qianji check` must evaluate the guard graph against the current artifact
state.

This includes the currently implemented guard categories and may expand in
future revisions to additional guard nodes.

A separate user-facing `plan-track` subcommand is not required. `plan-track`
is the guard face already implied by `qianji check`.

If any required guard node fails:

1. the current bounded slice must not be accepted
2. downstream done gates must remain blocked
3. Qianji must emit precise diagnostics
4. Codex must repair the artifact state and re-run `qianji check`

The accepted final state is therefore not "whatever Codex last wrote." The
accepted final state is "whatever passes the required guard graph."

### 10.4 Minimal Demo Graph

A representative target graph for the current lane may be expressed as:

```mermaid
flowchart LR
  coding --> rust
  rust --> style
  rust --> engineering_requirement
  rust --> policy
  rust --> blueprint_surface
  blueprint_surface --> plan_surface

  flowchart --> flowchart_alignment
  blueprint_surface --> flowchart_alignment
  plan_surface --> flowchart_alignment

  blueprint_surface --> boundary_check
  plan_surface --> boundary_check

  blueprint_surface --> drift_check
  plan_surface --> drift_check

  plan_surface --> status_legality

  flowchart_alignment --> done_gate
  boundary_check --> done_gate
  drift_check --> done_gate
  status_legality --> done_gate
```

This graph is a target-model illustration only. It must not be read as a
claim that every node above is already implemented in the current CLI or
represented as a live checked-in Flowhub directory.

### 10.5 Done Gate Semantics

A `done_gate` is the acceptance node for one bounded slice.

Its role is simple:

1. it decides whether the current slice may be treated as complete
2. it evaluates guard state rather than model confidence
3. it must remain closed until the required upstream guards pass

For first-version semantics, every guard node wired into `done_gate` must be
treated as required.

This means:

1. no separate optional-guard syntax is required in version one
2. if a guard is connected to `done_gate`, it is part of the acceptance
   contract
3. optional guards, if needed later, should be added by a future RFC

A first-version guard evaluation should recognize three states:

1. `pass`
2. `fail`
3. `blocked`

`pass` means the guard evaluated successfully and its condition is satisfied.

`fail` means the guard evaluated successfully and its condition is not
satisfied.

`blocked` means the guard could not yet be meaningfully evaluated because a
required upstream artifact or prerequisite guard is missing or unresolved.

A `done_gate` must pass if and only if all required upstream guard nodes are
in `pass`.

If any required upstream guard is `fail` or `blocked`, the `done_gate` must
not pass.

Therefore:

1. `pass` means the bounded slice is accepted
2. `fail` or `blocked` means the bounded slice is not accepted

### 10.6 Blocked vs Failed Diagnostics

`qianji check` must distinguish between a guard that failed and a guard that
is blocked.

These two states are not equivalent.

`fail` means the guard was evaluated and its condition was not satisfied.

`blocked` means the guard could not yet be meaningfully evaluated because a
required prerequisite was absent, unresolved, or not yet valid.

This distinction is required because repair action depends on it.

A guard is `fail` when:

1. the target artifact exists
2. the guard could run
3. the result is negative

A guard is `blocked` when:

1. the guard depends on a missing artifact, or
2. the guard depends on an upstream guard that did not pass, or
3. the guard cannot yet be evaluated with meaningful semantics

For both `fail` and `blocked`, Qianji must emit Codex-facing markdown
diagnostics.

Each diagnostic must state:

1. `Guard`
2. `Location`
3. `State`
4. `Problem`
5. `Why it blocks`
6. `Fix`

Failed example:

```md
# Validation Failed

## Flowchart Alignment

Guard: flowchart_alignment
Location: <plan-workdir>/flowchart.mmd
State: fail
Problem: `flowchart.mmd` does not express the required `blueprint -> plan` backbone
Why it blocks: the graph companion conflicts with the bounded artifact surface
Fix: rewrite `flowchart.mmd` so the visible backbone matches the current work surface
```

Blocked example:

```md
# Validation Failed

## Flowchart Alignment

Guard: flowchart_alignment
Location: <plan-workdir>/flowchart.mmd
State: blocked
Problem: `flowchart.mmd` is missing, so alignment cannot be evaluated
Why it blocks: the required graph companion surface does not yet exist
Fix: create `flowchart.mmd` before re-running `qianji check`
```

`done_gate` diagnostics should be derived rather than primary.

That means:

1. Qianji should report the failing or blocked upstream guards first
2. `done_gate` may be summarized afterward as blocked
3. `done_gate` should not be the only diagnostic when a more specific
   upstream guard explains the problem

Codex should interpret diagnostics as follows:

1. `blocked` means "create or restore the prerequisite surface first"
2. `fail` means "the surface exists, but its content or relation must be
   corrected"

This preserves a deterministic repair order:

1. resolve blocked guards first
2. re-run evaluation
3. repair failed guards
4. re-run evaluation
5. accept only when `done_gate` passes

## 11. Open Work For Next Revision

### 11.1 Minimal `qianji.toml` Field Set

The next revision should define the minimum field set for `qianji.toml` in
the bounded plan work surface.

### 11.2 `plan-track` Check Surface

The next revision should define the minimum boundary, drift, status, and
legality checks.

### 11.3 CLI Subcommand Contract

The next revision should standardize the exact `show` and `check` argument
contracts, along with the materialization entrypoint that creates the bounded
work surface.

## 12. Flowhub Contract

### 12.1 Flowhub-Root `qianji.toml`

The live Flowhub root is anchored by its own `qianji.toml`.

Minimal example:

```toml
version = 1

[flowhub]
name = "qianji-flowhub"

[contract]
register = ["coding", "rust", "blueprint", "plan"]
required = ["*/qianji.toml"]
```

Semantics:

1. `flowhub.name` is the stable library identifier
2. `contract.register` declares the allowed top-level graph-node directories
3. `contract.required` declares the required filesystem surfaces for those
   registered nodes
4. a `contract.required` entry beginning with `*/` expands once per registered
   child directory

This means the root contract is both:

1. a required-surface declaration
2. a directory allowlist

Therefore, a top-level directory that is not declared by `contract.register`
and not implied by `contract.required` is structural drift and must fail under
`qianji check`.

### 12.2 Module-Root `qianji.toml`

A module-root `qianji.toml` is the public contract for one Flowhub node.

Minimal leaf example:

```toml
version = 1

[module]
name = "rust"
tags = ["planning", "coding", "rust"]

[exports]
entry = "task.rust-start"
ready = "task.constraints-ready"
```

Semantics:

1. `module.name` is the stable node identifier
2. `module.tags` are metadata only
3. `exports` declare stable graph-facing handles

A leaf module may contain only that anchor manifest. It does not own child
graph directories or immediate local scenario-case files unless it explicitly
declares a local `[contract]`.

### 12.3 Optional Module `[contract]`

When a module owns child graph-node directories or immediate local surfaces,
it must declare them explicitly through `[contract]`.

Minimal composite example:

```toml
version = 1

[module]
name = "rust"
tags = ["planning", "coding", "rust"]

[exports]
entry = "task.rust-start"
ready = "task.constraints-ready"

[contract]
register = ["style", "engineering-requirement", "policy"]
required = ["*/qianji.toml"]
```

Semantics:

1. `contract.register` declares the allowed immediate child directories owned
   by that module
2. `contract.required` declares required surfaces relative to the module root
3. undeclared child directories under that module are structural drift

The contract is therefore the structure authority for both the Flowhub root
and any module that really owns child graph directories or immediate local
surfaces.

### 12.4 Contract Grammar

Version-one `[contract]` uses two fields:

1. `register`
2. `required`

#### `contract.register`

`contract.register` declares child graph-node references relative to the
current manifest directory.

Version-one rules:

1. every entry must be non-empty after trimming
2. every entry must stay relative
3. entries must not contain empty path segments
4. entries must not contain `.` or `..` segments
5. entries must not contain glob syntax such as `*`, `?`, `[` or `]`
6. duplicate entries are invalid

For the Flowhub root, `contract.register` must contain at least one entry.

#### `contract.required`

`contract.required` declares required filesystem surfaces relative to the
current manifest directory.

Version-one rules:

1. every entry must be non-empty after trimming
2. every entry must stay relative
3. entries must not contain empty path segments
4. entries must not escape the current node directory through `..`
5. duplicate entries are invalid

For the Flowhub root, `contract.required` must contain at least one entry.

`contract.required` may also own immediate Mermaid scenario-case files such as
`codex-plan.mmd` under a leaf node.

Version-one also allows one compact expansion rule:

1. an entry beginning with `*/` expands once per `contract.register` child

Validation example:

```toml
[contract]
register = ["coding", "rust", "blueprint", "plan"]
required = ["*/qianji.toml"]
```

This expands to:

1. `coding/qianji.toml`
2. `rust/qianji.toml`
3. `blueprint/qianji.toml`
4. `plan/qianji.toml`

### 12.5 Contract Drift Semantics

`qianji check` must evaluate `[contract]` as both:

1. a required-surface declaration
2. a child-directory allowlist

Therefore:

1. if a required path is absent, the contract fails
2. if a required glob matches nothing, the contract fails
3. if an immediate Mermaid scenario-case file is present but not declared by
   `contract.required`, the contract fails
4. if a child directory exists on disk but is not declared in
   `contract.register` and is not implied by `contract.required`, the contract
   fails as structural drift

This applies both:

1. at the Flowhub root
2. inside any module that declares a local `[contract]`

For leaf modules with no local `[contract]`, any child directory is drift.

### 12.6 Current Live Flowhub Rule

The current live Flowhub demo keeps only top-level node anchors:

1. `coding/`
2. `rust/`
3. `blueprint/`
4. `plan/`

In the current live tree, `rust` is a leaf node. It does not declare a local
`[contract]`, so any child directory under `rust/` is drift.

### 12.7 Contract Before Validation

`[contract]` is the primary structural contract surface.

`[[validation]]` remains available for additional module-specific checks, but
it does not replace `[contract]` as the authority on allowed child
directories and required filesystem surfaces.

### 12.8 Contract Diagnostics

When `[contract]` evaluation fails, `qianji check` must emit markdown
diagnostics rather than machine-only output.

At minimum, each diagnostic must include:

1. a short diagnostic title
2. `Location`
3. `Problem`
4. `Why it blocks`
5. `Fix`

Version-one Flowhub contract diagnostics should distinguish at least these
categories:

1. invalid root or module contract
2. missing required contract path
3. missing required contract glob matches
4. unregistered top-level directory drift
5. unregistered child-directory drift

Illustrative example:

```md
# Validation Failed

Location: <flowhub-root>
Checked modules: 1

## Unregistered child directory

Location: <flowhub-root>/rust/style
Problem: module `rust` contains child directory `style`, but it is not declared in `contract.register` and is not implied by `contract.required`
Why it blocks: the module graph shape has drifted away from its declared contract
Fix: add `style` to `contract.register` and `contract.required`, or remove the unregistered child directory
```

## 13. Additional `[[validation]]` Rules

`[[validation]]` rules are optional secondary checks declared in
`qianji.toml`.

They are used when a module or materialized work surface needs additional
rules beyond `[contract]`.

Version-one fields:

1. `scope`
2. `path`
3. `kind`
4. `required`
5. `min_matches`

Version-one scopes:

1. `module`
2. `scenario`

Version-one kinds:

1. `dir`
2. `file`
3. `glob`

### 13.1 `scope = "module"`

This validates a module-owned surface beyond the directory allowlist already
defined by `[contract]`.

Example:

```toml
[[validation]]
scope = "module"
path = "template/*.md"
kind = "glob"
min_matches = 1
```

### 13.2 `scope = "scenario"`

This validates the module after materialization into an active work surface.

```toml
[[validation]]
scope = "scenario"
path = "{alias}/qianji.toml"
kind = "file"
required = true

[[validation]]
scope = "scenario"
path = "{alias}/*.md"
kind = "glob"
min_matches = 1
```

`{alias}` resolves to the alias assigned in the active plan `[template].use`
declaration.

### 13.3 Interpretation

This contract means:

1. `[contract]` owns directory shape and required filesystem surfaces
2. `[[validation]]` augments that contract with additional checks
3. modules may still self-describe valid materialized instances
4. the CLI enforces structure without hardcoding every rule in global logic

## 14. Plan Composition Contract

The active plan-root `qianji.toml` has two responsibilities:

1. choose which flows to use
2. declare how those flows are linked

It must not restate module structure.

Minimal example:

```toml
version = 1

[planning]
name = "coding-rust-blueprint-plan-demo"
tags = ["planning", "coding", "rust", "demo"]

[template]
use = [
  "coding/rust as rust",
  "blueprint as blueprint",
  "plan as plan",
]

[[template.link]]
from = "rust::task.constraints-ready"
to = "blueprint::task.blueprint-start"

[[template.link]]
from = "blueprint::task.blueprint-ready"
to = "plan::task.plan-start"
```

Semantics:

1. `[planning]` stores plan metadata
2. `[template].use` selects Flowhub flows and assigns aliases
3. `[[template.link]]` defines cross-module graph edges

## 15. Materialization Semantics

Materialization converts the selected plan flow into a normalized Codex
operational work surface. Qianji should execute these steps:

1. read the active plan-root `qianji.toml`
2. parse `[template].use`
3. resolve selected modules in Flowhub
4. read each module-root `qianji.toml`
5. run `scope = "module"` validation
6. read each module `template/`
7. materialize the template under its alias
8. merge module graphs
9. apply `[[template.link]]`
10. emit the materialized work surface with root `qianji.toml`, root
    `flowchart.mmd`, `blueprint/`, and `plan/`
11. run `scope = "scenario"` validation
12. run any work-surface-level validation
13. enter the normal work loop only after validation passes

## 16. Validation Contract

Validation is the primary correctness mechanism. By default it does not repair
anything. Its job is to emit:

1. exact location
2. exact problem
3. exact reason
4. exact next action

The default output surface is markdown diagnostics, not JSON-first output and
not opaque error codes.

### 16.1 Unified `qianji check` Markdown Contract

The markdown output contract for `qianji check` should stay structurally
stable across:

1. Flowhub root and module checks
2. Flowhub scenario checks
3. bounded work-surface checks

For a passing check, the output should begin with:

```md
# Validation Passed
```

and may then include context-specific summary lines such as plan name,
scenario name, location, checked-module count, or visible surfaces.

For a failing check, the output should begin with:

```md
# Validation Failed
```

and each diagnostic section must preserve this markdown skeleton:

1. section title
2. `Location`
3. `Problem`
4. `Why it blocks`
5. `Fix`

The exact titles may vary by surface, but the markdown shape should remain the
same so Codex can reliably read and react to diagnostics across all supported
`qianji check` targets.

### 16.2 Unified `qianji show` Markdown Contract

The markdown output contract for `qianji show` should stay structurally stable
across:

1. Flowhub root and module previews
2. Flowhub scenario previews
3. bounded work-surface previews

The first line should always be a single H1 title naming the current target
surface.

After that H1, the render should preserve two layers:

1. metadata lines such as plan name, scenario name, location, Flowhub root,
   module count, or other first-order context
2. H2 sections for each visible first-order surface, module, or preview block

The exact H1 titles may vary by target, but the markdown skeleton should
remain stable so Codex can scan all supported `qianji show` surfaces the same
way.

Example:

```md
# Work Surface

Plan: demo-plan
Location: <plan-workdir>

## flowchart.mmd

Path: <plan-workdir>/flowchart.mmd
Status: file

## blueprint

Path: <plan-workdir>/blueprint
Status: directory
Entries:

- architecture.md
```

Example:

```md
# Validation Failed

## Missing materialized template manifest

Location: <active-plan-work-surface>/plan
Problem: materialized plan work surface does not contain flowchart.mmd
Why it blocks: Codex and human operators do not have the required graph entry surface
Fix: ensure the `plan` flow emits `flowchart.mmd` at the work-surface root during materialization
```

Validation must at least cover:

1. bundle structure validity
2. module resolution
3. `[template].use` resolution
4. module `[[validation]]` contract satisfaction
5. export resolution
6. `[[template.link]]` resolution
7. graph compile readiness after materialization
8. flowchart alignment against the active plan graph
9. boundary, drift, and status legality checks
10. structural heading/property/depends rules when those contracts are defined

## 17. Ownership Alignment

This RFC assigns responsibilities as follows:

1. Flowhub owns reusable planning flows
2. the active plan-root manifest owns flow selection and graph linking
3. Qianji owns flow resolution, plan-work-surface materialization, plan-track
   validation orchestration, graph compile, and runtime handoff
4. Zhenfa owns reusable contract and validation-library logic
5. Wendao owns exact SQL-based fragment query and context retrieval

This is consistent with current repository direction, but it remains a draft
planning model until the corresponding schema and CLI slices land.

## 18. First Stable Slice

Version one only requires these surfaces to become stable:

1. `qianji-flowhub/plan`
2. the derived `plan-track` subgraph
3. the active Codex operational workdir
4. the active plan-root `qianji.toml`
5. root `flowchart.mmd`
6. `blueprint/`
7. `plan/`
8. module-root `[[validation]]`
9. `[template].use`
10. `[[template.link]]`
11. CLI-first `show` and `check` flow
12. markdown validation diagnostics

## 19. Open Questions

Future RFCs should settle:

1. the minimum required field set for the bounded work-surface `qianji.toml`
2. whether the active plan-root `[template]` should support tag-based module
   selection or remain explicit-name-only in version one
3. the minimum required fields for template-local `qianji.toml`
4. whether `[[validation]]` should grow beyond path structure into
   heading/property/depends contracts
5. the exact `show` and `check` argument grammar and output contract
6. whether alias directories must always be written to disk or may remain
   partially virtualized

## 20. Conclusion

This RFC proposes a Flowhub graph-contract model built on:

1. `qianji-flowhub/plan` as the source of materialized plan work surfaces
2. `plan-track` as the derived tracking subgraph
3. the Codex operational workdir as the primary bounded execution surface
4. the minimal bounded work surface of `qianji.toml`, `flowchart.mmd`,
   `blueprint/`, and `plan/`
5. module-root `[[validation]]` as the source of structural truth
6. a CLI-first `show` and `check` control plane for Codex
7. validation as a precise diagnostic gate
8. Wendao SQL-based exact fragment retrieval plus structural compression as
   the normal
   context-retrieval strategy

The point is not to make the model write everything correctly in one pass.
The point is to make plan generation bounded, execution local, reading
precise, validation deterministic, and Codex consistently confined to one
small operational surface.
