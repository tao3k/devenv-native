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
   bounded parallel multi-instance service tasks, DMN rule tables, or later
   supported constructs.
3. Run `qianji construct index` to inspect the available construct table of
   contents.
4. Run `qianji construct show <id>` only for the selected constructs.
5. Fill the BPMN or DMN scaffold from the selected cards.
6. Run `qianji lint <workflow.bpmn>` or `qianji lint <decision.dmn>`.
7. Repair from the compact LLM lint diagnostics until the executable artifact
   passes.

For user interactions, the active native BPMN IO contract accepts only
`input`, `confirm`, `choice`, and `choice_input` interactionType literals.
Use `input` for plain free-form answers and `choice_input` when the checkpoint
needs option selection plus optional feedback text. Static choices use a JSON
array assignment on dataInput `choices`. Dynamic choices use a
`dataInputAssociation/sourceRef` for dataInput `choices`, where an upstream
service task writes `currentChoices` as structured JSON choice objects with
required `value` fields instead of embedding option text in `currentQuestion`.
`qianji lint --llm` reports legacy non-native interaction XML and proposes the
native BPMN IO contract.
When the question and choices are fixed at compile time, declare them directly
on the `userTask`; `qianji lint --llm` reports no-input/no-tool producer
`serviceTask` nodes that only prepare fixed interaction metadata.
When choices are truly dynamic, the producer `serviceTask` must explicitly
bind every declared data input name in its documentation prompt. A producer
that declares runtime inputs but does not mention those input variables is
treated as an unbound UI-metadata producer; `qianji lint --llm` reports it so
the compiler either inlines fixed JSON choices on the `userTask` or rewrites
the producer prompt to bind the runtime inputs by name.

User answers are persisted through the `dataOutput name="answer"` mapping.
Do not insert no-tool `serviceTask` nodes that only store, copy, or rename that
answer before the next prompt. `qianji lint --llm` reports redundant
user-answer store serviceTasks and asks the compiler to remove the store node,
reconnect the userTask to the next task, and replace downstream data-input
aliases with the original answer variable. Keep a serviceTask only when it
derives route booleans, summaries, decisions, or tool-backed outputs that are
not already the user answer.

For serviceTask tool scope, keep BPMN XML limited to standard documentation
and native IO metadata. Host capability policy belongs in the host adapter
contract, not in custom BPMN XML. Declared data inputs are injected as
read-only workflow variables, so reading `specContent` or another declared
input does not justify shell or filesystem capabilities inside the BPMN file.

For gateway routing, align condition syntax with the runtime value type. A bare
condition path such as `approved` must resolve to a JSON boolean. A count-like
value such as `questionsRemaining` must use a numeric comparison such as
`questionsRemaining > 0`, or be renamed to a boolean-shaped output such as
`hasMoreQuestions`.

The current authoring loop is:

```sh
cat ~/.agents/skills/brainstorming/SKILL.md
qianji construct index
qianji construct show user-task.interaction
qianji construct show gateway.exclusive.bounded
qianji lint workflow.bpmn
```

Discovery commands and lint accept `--json` when an SDK or compiler needs
structured data. The default lint output is the compact repair diagnostic
surface for LLM observations:

```sh
qianji construct index --json
qianji construct show gateway.exclusive.bounded --json
qianji lint workflow.bpmn --json
```

For LLM repair loops, prefer the compact text diagnostic over the JSON report.
The text surface keeps natural language in the diagnostic layer and keeps the
repair layer as a git-diff-style patch:

- diagnostic layer: code, title, file span, source line, caret label, one-line
  `Help`, and optional one-line `Contract`
- repair layer: `Proposed patch` with `---`, `+++`, and git-style hunk
  headers such as `@@ -line,count +line,count @@`
- output constraint: `Return unified diff only.`

Do not reintroduce verbose `Action`, `Patch focus`, or `Structured repair`
sections into the LLM text output. Those details belong in structured JSON for
tools that need them.

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
- `service-task.multi-instance.parallel`
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
