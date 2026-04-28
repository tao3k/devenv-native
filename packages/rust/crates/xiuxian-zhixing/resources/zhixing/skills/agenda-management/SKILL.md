---
kind: SKILL.md
type: skill
title: "Agenda Management"
category: "skills"
tags:
  - synaptic-flow
  - gtd
  - socratic-audit
name: agenda-management
description: High-fidelity scheduling and cognitive alignment via the Triangular Synaptic Flow.
author: CyberXiuXian
date: 2026-04-26T09:30-07:00
metadata:
  retrieval:
    saliency_base: 5.5
    decay_rate: 0.05
  version: "1.1.0"
  source: "https://github.com/tao3k/xiuxian-artisan-workshop/tree/main/packages/rust/crates/xiuxian-zhixing/resources/zhixing/skills/agenda-management"
  routing_keywords:
    - "agenda"
    - "schedule"
    - "task planning"
    - "procrastination audit"
  intents:
    - "Draft a new schedule"
    - "Audit my productivity"
    - "Resolve task conflicts"
---

# Skill Manifest: Agenda Management

This skill implements the **Triangular Synaptic Architecture** to ensure that daily execution is physically grounded and cognitively aligned.

## 1. The Knowledge Fortress

The Trinity utilizes these foundational frameworks to govern the workshop:

- [Knowledge Fortress](references/methodologies.md)

## 2. The Trinity (Artisan Souls)

- **Student**: [Student persona](references/student.md) (The Ambitious Aspirant)
- **Steward**: [Steward persona](references/steward.md) (The Clockwork Guardian)
- **Professor**: [Professor persona](references/teacher.md) (The Sage of Alignment)

## 3. The Synaptic Flow (Execution)

The adversarial negotiation is governed by:

- [Agenda flow](references/agenda_flow.toml)

## 4. Manifestation Templates

- **Drafting**: [Draft agenda template](references/draft_agenda.j2)
- **Critique**: [Critique agenda template](references/critique_agenda.j2)
- **Final Reflection**: [Final agenda template](references/final_agenda.j2)

## 5. Registry Anchors

These anchors expose stable IDs for Wendao parser/indexer contracts.

### Steward Persona

<!-- id: "steward", type: "persona", target: "references/steward.md" -->

- [Steward persona](references/steward.md)

### Teacher Persona

<!-- id: "teacher", type: "persona", target: "references/teacher.md" -->

- [Teacher persona](references/teacher.md)

### Rules Knowledge

<!-- id: "rules", type: "knowledge", target: "references/rules.md" -->

- [Rules knowledge](references/rules.md)

### Agenda Classifier Prompt

<!-- id: "agenda_classifier", type: "knowledge", target: "references/prompts/classifier.md" -->

- [Agenda classifier prompt](references/prompts/classifier.md)

### Agenda Validation Genesis Rules

<!-- id: "agenda_validation_genesis_rules", type: "knowledge", target: "references/prompts/agenda_validation_genesis_rules.md" -->

- [Agenda validation genesis rules](references/prompts/agenda_validation_genesis_rules.md)

### Agenda Flow

<!-- id: "agenda_flow", type: "workflow", target: "references/agenda_flow.toml" -->

- [Agenda flow](references/agenda_flow.toml)

### Draft Template

<!-- id: "draft_agenda.j2", type: "template", target: "references/draft_agenda.j2" -->

- [Draft agenda template](references/draft_agenda.j2)

### Critique Template

<!-- id: "critique_agenda.j2", type: "template", target: "references/critique_agenda.j2" -->

- [Critique agenda template](references/critique_agenda.j2)

### Final Template

<!-- id: "final_agenda.j2", type: "template", target: "references/final_agenda.j2" -->

- [Final agenda template](references/final_agenda.j2)
