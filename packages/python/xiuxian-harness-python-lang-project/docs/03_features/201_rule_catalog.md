# Rule Catalog

:PROPERTIES:
:ID: 4af1b8e663ad854b94d6bd438596ab03c9829653
:TYPE: FEATURE
:STATUS: ACTIVE
:LAST_SYNC: 2026-04-29
:END:

The harness exposes deterministic rule metadata as compact library data through
`python_modern_design_rules()`. This keeps agents and pytest snapshots focused
on stable rule contracts instead of large JSON payloads.

## Default Rule Packs

Default harness execution runs:

1. `python.syntax`: blocks invalid CPython-native syntax before design rules
   run
2. `python.modern_design`: emits numbered modern Python design findings

## Modern-Design Rules

Current numbered rules:

1. `PY-MOD-R001`: wildcard imports must be replaced with explicit imported
   names
2. `PY-MOD-R002`: bare `print` calls are not allowed in library modules
3. `PY-MOD-R003`: package facade re-exports must declare explicit `__all__`
4. `PY-MOD-R004`: bare `breakpoint()` calls are not allowed in library modules

## Catalog API

```python
from xiuxian_harness_python_lang_project import python_modern_design_rules

assert [rule.rule_id for rule in python_modern_design_rules()] == [
    "PY-MOD-R001",
    "PY-MOD-R002",
    "PY-MOD-R003",
    "PY-MOD-R004",
]
```

Each catalog entry includes:

1. rule id
2. pack id
3. severity
4. title
5. requirement
6. labels

## Rendered Diagnostic Policy

Rendered findings intentionally avoid why/fix/evidence blocks. They show:

1. stable rule id
2. source location
3. highlighted source line
4. short label
5. one precise `Required:` contract line

This compact shape is the primary repair surface for agents. Dictionary export
exists for explicit tooling, but JSON is not the default repair medium.

:RELATIONS:
:LINKS: [Harness Boundary](../01_core/101_harness_boundary.md)
:END:
