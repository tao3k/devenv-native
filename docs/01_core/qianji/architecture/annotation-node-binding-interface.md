---
type: knowledge
title: "Qianji Annotation Node Binding Specification"
category: "architecture"
tags:
  - qianji
  - annotation
  - interface
  - toml
saliency_base: 8.0
decay_rate: 0.02
metadata:
  title: "Qianji Annotation Node Binding Specification"
---

# Qianji Annotation Node Binding Specification

This specification defines the local Qianji data contract for node-level
annotation snapshots.

The serialized TOML table is `[nodes.annotation]`, and the runtime owner is
Qianji-local annotation code rather than a separate prompt-persona crate.

## 1. TOML Schema

```toml
[[nodes]]
id = "Strict_Auditor_Node"
task_type = "annotation"
weight = 1.0

  [nodes.annotation]
  persona_id = "strict_architecture_auditor"
  template_target = "critique_report.j2"
  execution_mode = "isolated"
  input_keys = ["proposer_node.output_xml"]
  history_key = "annotation_history"
  output_key = "annotated_prompt"
```

## 2. Runtime Contract

When a Qianji annotation node executes:

1. Qianji resolves `persona_id`, `template_target`, and `input_keys` against
   the current workflow context.
2. It gathers only the declared input keys. `isolated` mode does not gather
   prior history. `appended` mode reads and updates `history_key`.
3. It resolves `wendao://` persona references through the Wendao runtime when
   used; plain persona ids become local deterministic role profiles.
4. It renders one escaped XML snapshot under the configured `output_key`.
5. It records metadata keys for persona id, execution mode, input keys, history
   key when applicable, and template target when configured.

This keeps annotation deterministic and local to the workflow runtime.
