# Qianji Construct Cards

Qianji construct cards are a progressive-disclosure surface for LLM workflow
compilers. They replace the need to load a large BPMN or DMN template before
the model understands which executable constructs are relevant.

The intended agent flow starts from a source task or `SKILL.md`. The source
file is semantic input, not a WorkflowPlan by definition. The LLM decides
whether the source can become an autonomous workflow, an interactive workflow,
or a planning workflow that must ask the user before execution.

The intended compiler loop is:

1. Read the task or skill source.
2. Classify the scenario shape: autonomous, interactive, or planning-first;
   then map that scenario to agent tasks, user interactions, bounded gateways,
   DMN rule tables, or later supported constructs.
3. Run `qianji construct index` to inspect the available construct table of
   contents.
4. Run `qianji construct show <id>` only for the selected constructs.
5. Fill the BPMN or DMN scaffold from the selected cards.
6. Run `qianji lint <workflow.bpmn>` or `qianji lint <decision.dmn>`.
7. Repair from lint diagnostics until the executable artifact passes.

For user interactions, the active qianji extension contract accepts only
`input`, `confirm`, `choice`, and `choice_input` as `qianji:interaction`
types. Use `input` for plain free-form answers and `choice_input` when the
checkpoint needs option selection plus optional feedback text.

The current authoring loop is:

```sh
cat ~/.agents/skills/brainstorming/SKILL.md
qianji construct index
qianji construct show user-task.interaction
qianji construct show gateway.exclusive.bounded
qianji lint workflow.bpmn
```

Discovery commands accept `--json` when an SDK or compiler needs structured
data:

```sh
qianji construct index --json
qianji construct show gateway.exclusive.bounded --json
```

Each card includes:

- a stable construct id
- domain and lifecycle status
- purpose and selection guidance
- required neighboring contracts
- allowed bounded forms
- forbidden anti-patterns
- a minimal BPMN or DMN scaffold for the selected construct
- lint diagnostic repair guidance
- related cards

## Boundary

Construct semantics belong to Qianji. Downstream tools such as pi-wendao may
consume the catalog, but they should not maintain a forked copy of the engine
contract. This keeps lint diagnostics, examples, and future WorkflowPlan
validation anchored to the same package that owns the BPMN/DMN runtime.

## Current Seed Cards

- `service-task.agent`
- `user-task.interaction`
- `gateway.exclusive.bounded`
- `dmn.decision-table.unique`

## Optional WorkflowPlan Boundary

Construct cards do not require the source Markdown to be a WorkflowPlan and do
not make WorkflowPlan JSON the primary card output. A compiler may still use a
WorkflowPlan as an internal lowering IR when it wants deterministic BPMN
emission through `qianji emit <plan.json> --bpmn`.

`qianji lint <plan.json>` checks version 1 JSON plans with selected
constructs, executable tasks, and edges:

```json
{
  "version": 1,
  "name": "approval-plan",
  "constructs": [
    "service-task.agent",
    "user-task.interaction",
    "gateway.exclusive.bounded"
  ],
  "tasks": [
    {
      "id": "Task_Check",
      "construct": "service-task.agent",
      "outputs": ["ready"]
    },
    {
      "id": "Task_Approve",
      "construct": "user-task.interaction",
      "inputs": ["ready"],
      "outputs": ["approved"]
    }
  ],
  "edges": [
    { "from": "start", "to": "Task_Check" },
    { "from": "Task_Check", "to": "Task_Approve" },
    { "from": "Task_Approve", "to": "end", "condition": "approved" }
  ]
}
```

If a compiler uses this IR, it should not wrap it inside a `plan` object,
should not use `nodes`, and should keep `version` as the numeric value `1`.
Treat `constructs` as a set of selected construct ids: include each construct
once even when multiple tasks use the same construct. `tasks` may contain
executable work only: service tasks and user interactions. Gateway constructs
are selected in `constructs`, but represented by conditional/default `edges`.

Validation and emission are static. They do not execute the workflow.
