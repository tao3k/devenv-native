---
type: knowledge
metadata:
  title: "Journal Captured"
---

# Journal Captured

{% if manifestation.persona and manifestation.persona.name %}

> Steward: **{{ manifestation.persona.name }}**
> {% endif %}
> {{ manifestation.injected_context }}

- Manifested Task: **{{ task_title }}**
- Task ID: `{{ task_id }}`
- Journal ID: `{{ journal_id }}`

Next step: run `agenda.view` to verify execution order and time slots.
