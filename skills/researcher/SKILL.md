---
kind: SKILL.md
type: skill
title: "Repository Researcher"
category: "workflows"
tags:
  - research
  - repository-analysis
  - architecture-review
name: researcher
description: Use when analyzing repositories, conducting deep research on codebases, performing architecture reviews, or exploring large projects from a git URL.
author: xiuxian-artisan-workshop
date: 2026-04-26T09:30-07:00
metadata:
  retrieval:
    saliency_base: 5.5
    decay_rate: 0.05
  version: "2.1.0"
  source: "https://github.com/tao3k/xiuxian-artisan-workshop/tree/main/assets/skills/researcher"
  routing_keywords:
    - "research"
    - "analyze"
    - "analyze_repo"
    - "deep_research"
    - "code_analysis"
    - "repository_map"
    - "sharded_analysis"
    - "architecture_review"
    - "git"
    - "repo"
    - "repository"
    - "github"
    - "url"
  intents:
    - "Research repository"
    - "Analyze codebase"
    - "Deep research"
    - "Architecture review"
    - "Analyze git repo or link"
---

# Researcher Skill

Sharded deep research for large repositories using the **Qianji runtime** (`xiuxian-qianji`) and a suspend/resume approval loop.

## Architecture

```
┌─────────────┐     ┌────────────────┐     ┌──────────────────┐
│   Setup     │ --> │ Architect Plan │ --> │ Await Approval   │
│ clone + map │     │ shard proposal │     │ suspend/resume   │
└─────────────┘     └────────────────┘     └──────────────────┘
                                                    │
                                                    ▼
                                          ┌──────────────────┐
                                          │ Deep Analysis    │
                                          │ approved shards  │
                                          └──────────────────┘
```

## Command

### `git_repo_analyer`

Core command to execute repository research via Qianji.

Parameters:

- `repo_url` (string, required): Git repository URL to analyze.
- `request` (string, optional): Research goal. Default: `"Analyze the architecture"`.
- `action` (string, optional): `"start"` or `"approve"`. Default: `"start"`.
- `session_id` (string, required for `approve`): Session returned by `start`.
- `approved_shards` (string, required for `approve`): Approved plan JSON string.

Execution model:

1. `action="start"`:
   - clones and maps repository,
   - asks architect to propose shard plan,
   - returns `session_id`, `proposed_plan`, and approval prompt.
2. `action="approve"`:
   - resumes same session with approved shard JSON,
   - runs deep analysis for approved shards,
   - returns final analysis payload.

## Output

The command returns structured JSON. Typical fields:

- `success`
- `session_id`
- `message` / `proposed_plan` (start phase)
- `analysis_result` / `full_context` (approve phase)

## Implementation Notes

- Runtime backend is `xiuxian-qianji` (Rust).
- Workflow definition is `workflows/repo_analyzer.toml`.
- No Python local runtime is maintained under this skill.

## Files

```
researcher/
├── SKILL.md
├── README.md
├── references/
│   └── run_research_graph.md
├── workflows/
│   └── repo_analyzer.toml
```
