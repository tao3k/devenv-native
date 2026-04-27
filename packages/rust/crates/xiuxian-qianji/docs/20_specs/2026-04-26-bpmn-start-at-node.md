# BPMN Start-At Node

`qianji bpmn start-at` creates a fresh BPMN workflow instance with the first
active token placed at one selected BPMN node:

```bash
qianji bpmn start-at \
  --bpmn workflow.bpmn \
  --process Process_1 \
  --node Task_Question \
  --instance-id wf_start_at_question \
  --context-json '{"currentQuestion":"What should we explore?"}' \
  --external-host \
  --trace-stream
```

This is a test and debugging entrypoint. It is not a checkpoint mutation API.
If the instance id already has a checkpoint, `start-at` fails and the caller
must either resume, cancel, or choose a fresh instance id.

## Supported Nodes

The first slice supports host-bound task entry:

- `serviceTask`
- `scriptTask`
- `businessRuleTask`
- `userTask`
- `manualTask`
- `sendTask`

Gateways, events, call activities, and subprocess internals are intentionally
unsupported until the runtime has explicit contracts for synthetic gateway
state, event subscriptions, and parent frame construction.

## Ownership Boundary

Qianji owns synthetic session construction, checkpoint collision checks, and
scheduler execution. Hosts such as pi-wendao may expose a convenience flag, but
they must not construct or mutate qianji checkpoints directly.
