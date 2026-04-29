# xiuxian-harness-python-lang-project

`xiuxian-harness-python-lang-project` is a project-level Python language
harness library for modern Python projects.

It depends on `python-lang-parser` for Python-native AST and symbol-table
parsing primitives, then adds project discovery, rule packs, compact rendered
diagnostics, and pytest-friendly blocking assertions. It does not ship a CLI.

## Quick Use

```python
from pathlib import Path

from xiuxian_harness_python_lang_project import assert_python_lang_harness_clean


def test_python_language_harness() -> None:
    assert_python_lang_harness_clean([Path("src")])
```

## Docs

Detailed package material lives under [`docs/`](docs/index.md):

- [Harness Boundary](docs/01_core/101_harness_boundary.md)
- [Rule Catalog](docs/03_features/201_rule_catalog.md)

Use `wendao audit --template johnny-decimal` as the authoring template entry
when expanding this documentation surface. Keep this README as a compact
package entrypoint.

## Current Rule Packs

- `python.syntax`
- `python.modern_design`

Rendered findings stay compact: rule id, source location, highlighted source
line, short label, and one precise `Required:` contract line.

The current numbered modern-design rules are documented in
[Rule Catalog](docs/03_features/201_rule_catalog.md).
