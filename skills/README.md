---
type: knowledge
metadata:
  title: "Skills Directory"
---

# Skills Directory

> scripts/commands.py Pattern - No tools.py Required

This directory contains **Skills** - composable, self-contained packages that provide specific capabilities to the Xiuxian Daochang.

## Quick Reference

| Topic             | Documentation                                 |
| ----------------- | --------------------------------------------- |
| Creating a skill  | [Creating a New Skill](#creating-a-new-skill) |
| Architecture      | [Skill Structure](#skill-structure)           |
| Command reference | See individual skill `SKILL.md` files         |

## Skill Structure

```
assets/skills/{skill_name}/
├── SKILL.md           # Metadata + LLM context (YAML frontmatter)
├── scripts/           # Commands (registered runtime functions)
│   ├── __init__.py    # Dynamic module loader (importlib.util)
│   └── commands.py    # All skill commands
├── README.md          # Human-readable documentation
├── templates/         # Jinja2 templates (cascading pattern)
├── references/        # Per-tool or per-skill docs (YAML: metadata.for_tools)
└── tests/             # Test files
```

**Data hierarchy:** `SKILL.md` is the **top-level comprehensive** doc for the skill; **tools** come only from `scripts/`; **references/** hold detailed docs. In each reference markdown use frontmatter: `metadata.for_tools: <skill.command>` (and optionally `metadata.title`). See [Skill Data Hierarchy and References](../../docs/reference/skill-data-hierarchy-and-references.md).

## Runtime Surface

Historical note: older revisions used a dedicated Python protocol server. The
current repository keeps skills as documentation-plus-script packages and lets
the retained runtime/tool surfaces own transport concerns.

```python
# runtime_entry.py - minimal runtime registration sketch
from some_tool_runtime import Runtime

runtime = Runtime("xiuxian-daochang")

@runtime.list_tools()
async def list_tools(): ...

@runtime.call_tool()
async def call_tool(name, arguments): ...
```

**Benefits:**

- Direct control over tool listing/execution
- Explicit error handling for TaskGroup
  嗯。- Optional uvloop (SSE mode) + orjson for performance
- No removed legacy protocol-server dependency overhead

## Cascading Templates

Skills support **cascading template loading** with "User Overrides > Skill Defaults" pattern:

```
skills/example/                       # Skill Directory
├── templates/                        # Skill defaults (Fallback)
│   ├── commit_message.j2
│   ├── workflow_result.j2
│   └── error_message.j2
└── scripts/
    ├── __init__.py                   # Package marker (required!)
    └── commands.py                    # registered commands

assets/templates/                      # User overrides (Priority)
└── git/
    ├── commit_message.j2              # Overrides skill default
    └── workflow_result.j2
```

**Template Resolution Order:**

1. `assets/templates/{skill}/` - User customizations (highest priority)
2. `assets/skills/{skill}/templates/` - Skill defaults (fallback)

## Creating a New Skill

### 1. Copy the Template

```bash
cp -r assets/skills/_template assets/skills/my_new_skill
```

### 2. Add Commands in scripts/commands.py

```python
from xiuxian_foundation.api.decorators import skill_command

@skill_command(
    name="my_command",
    category="read",
    description="Brief description of what this command does",
)
async def my_command(param: str) -> str:
    """Detailed docstring explaining the command behavior."""
    return "result"
```

**Note:** Command name is just `my_command`, not `my_new_skill.my_command`. The
runtime surface is responsible for any external namespacing.

## Command Categories

| Category   | Use Case                                      |
| ---------- | --------------------------------------------- |
| `read`     | Read/retrieve data (files, git status, etc.)  |
| `view`     | Visualize or display data (formatted reports) |
| `write`    | Create or modify data (write files, commit)   |
| `workflow` | Multi-step operations (complex tasks)         |
| `general`  | Miscellaneous commands                        |

## Command Registration

Command functions in `scripts/*.py` are registered as runtime-discoverable
tools:

```python
@skill_command(
    name="command_name",       # Tool name (required)
    category="read",           # Category from SkillCategory enum
    description="Brief desc",  # Tool description for LLM
)
async def command_name(param: str) -> str:
    """Function docstring becomes detailed description."""
    return "result"
```

## Hot Reload

Skills are automatically reloaded when retained skill files change. Mtime checks
are throttled to once per 100ms.

## Skill Metadata (SKILL.md)

Each skill has a `SKILL.md` with YAML frontmatter using Anthropic official format:

```yaml
---
type: skill
name: git
description: Use when working with version control, commits, branches, or Git operations.
metadata:
  author: xiuxian-artisan-workshop
  version: "2.0.0"
  source: "https://github.com/tao3k/xiuxian-artisan-workshop/tree/main/skills/git"
  routing_keywords:
    - "git"
    - "commit"
    - "push"
    - "branch"
  intents:
    - "hotfix"
    - "pr"
    - "commit"
    - "status"
---

# Git Skill

> **Code is Mechanism, Prompt is Policy**

## Status

The repository no longer maintains a Python local runtime under `skills/git`.
Retained skill directories may be metadata-only.
```

## Example Skills

| Skill                                           | Features                      |
| ----------------------------------------------- | ----------------------------- |
| [Git](./git/SKILL.md)                           | Metadata-only retired runtime |
| [Filesystem](./filesystem/SKILL.md)             | Read, write, search files     |
| [Terminal](./terminal/SKILL.md)                 | Shell command execution       |
| [Testing Protocol](./testing_protocol/SKILL.md) | Test runner integration       |

## Related Documentation

- [Skill Standard](../../docs/human/architecture/skill-standard.md) - Living Skill Architecture
- [Skill Lifecycle](../../docs/human/architecture/skill-lifecycle.md) - Workflow runtime support
- [Trinity Architecture](../../docs/explanation/trinity-architecture.md) - Technical deep dive
