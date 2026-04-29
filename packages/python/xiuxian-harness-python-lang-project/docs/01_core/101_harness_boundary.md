# Harness Boundary

:PROPERTIES:
:ID: 5fa0fe2dac2c4668b1ad949a8590d0098679cb1b
:TYPE: CORE
:STATUS: ACTIVE
:LAST_SYNC: 2026-04-29
:END:

`xiuxian-harness-python-lang-project` is a Python helper library for modern
Python projects. It depends on `python-lang-parser` for Python-native AST,
compiler, and symbol-table reports, then adds project discovery, rule-pack
evaluation, compact diagnostics, and pytest-friendly blocking assertions.

## Ownership

This package may:

1. discover Python source files in a project
2. evaluate parser diagnostics through library rule packs
3. emit deterministic harness findings
4. render compact source diagnostics for repair workflows
5. block pytest tests on error or warning findings
6. expose deterministic numbered Python design rules

This package must not own:

1. Python runtime orchestration
2. workflow execution
3. routing, memory, indexing, or transport
4. pytest itself
5. a command-line interface

## Native Parser Dependency

Python source structure comes from `python-lang-parser`. The parser uses
Python's own standard-library parser, compiler, and symbol-table surfaces,
currently `ast`, `compile`, `tokenize.open`, and `symtable`. It does not use
`tree-sitter`.

Third-party concrete-syntax parsers are deferred until a consumer needs
comments, exact whitespace, or formatting-preserving spans.

## Pytest Embedding

The primary integration is a pytest assertion:

```python
from pathlib import Path

from xiuxian_harness_python_lang_project import assert_python_lang_harness_clean


def test_python_language_harness() -> None:
    assert_python_lang_harness_clean([Path("src")])
```

The assertion raises `AssertionError` with the compact rendered diagnostic
report when error or warning findings are present.

:RELATIONS:
:LINKS: [Rule Catalog](../03_features/201_rule_catalog.md)
:END:
