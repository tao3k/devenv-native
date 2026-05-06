---
kind: SKILL.md
type: skill
title: "Git Version Control Operations"
category: "workflows"
tags:
  - git
  - version-control
  - commit
name: git
description: Use when committing code, managing branches, pushing to remote, creating pull requests, or performing version control operations. Conforms to docs/reference/skill-routing-value-standard.md.
author: xiuxian-artisan-workshop
date: 2026-04-26T09:30-07:00
saliency_base: 5.5
decay_rate: 0.05
metadata:
  version: "2.0.0"
  source: "https://github.com/tao3k/xiuxian-artisan-workshop/tree/main/skills/git"
  routing_keywords:
    - "git"
    - "commit"
    - "push"
    - "pull"
    - "merge"
    - "rebase"
    - "checkout"
    - "stash"
    - "tag"
    - "commit code"
    - "save changes"
    - "commit changes"
    - "push code"
    - "save work"
    - "check in"
    - "submit code"
    - "version control"
    - "branch"
    - "repo"
    - "repository"
    - "history"
    - "diff"
    - "status"
    - "log"
    - "pr"
    - "pull request"
    - "code review"
  intents:
    - "Create pull request"
    - "Manage branches"
    - "Commit code"
    - "Stash changes"
    - "Merge branches"
    - "Rebase branches"
    - "Tag commits"
    - "Check git status"
---

# Git Skill

`git` remains a routing keyword for git-related requests, but this repository
no longer maintains a Python local runtime under `skills/git`.

## Status

1. Python scripts, local pytest surfaces, and runtime-coupled templates were
   retired from this skill directory.
2. `skills/git` is now metadata- and knowledge-only.
3. Repository-local execution should use native git CLI flows or other
   app-owned/runtime-owned integrations outside `skills/`.

## Boundaries

1. Do not add new executable Python runtime files under `skills/git`.
2. Future git execution surfaces must live outside `skills/`.
3. Active documentation must not describe deleted local script paths as a live
   implementation path.

## Retained Files

```
git/
├── SKILL.md
├── README.md
├── assets/
│   └── Backlog.md
└── extensions/
    └── sniffer/rules.toml
```
