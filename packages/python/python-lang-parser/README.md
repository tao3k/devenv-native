# python-lang-parser

`python-lang-parser` is a small Python-native source parser package. It uses
Python's own standard-library parser and compiler-introspection surfaces,
currently `ast.parse`, `tokenize.open`, `compile`, and `symtable`, rather than
`tree-sitter`. It owns AST-backed parsing primitives, native symbol-table
bindings, reference and call-site indexes, and compact module reports for
modern Python projects.

The package intentionally does not own test discovery, rule packs, CLI
rendering, pytest integration, runtime orchestration, or project policy.
Project harness packages can depend on it to build higher-level diagnostics.

## Quick Use

```python
from python_lang_parser import parse_python_source

report = parse_python_source(
    "import pathlib\n\nclass Runner:\n    def run(self) -> None:\n        pass\n",
    path="runner.py",
)

assert report.is_valid
assert [symbol.qualified_name for symbol in report.symbols] == [
    "Runner",
    "Runner.run",
]
assert report.metadata["parser"] == "cpython.ast"
assert report.metadata["symbol_table"] == "cpython.symtable"
```

Reports are intentionally small enough to snapshot or render for Agent repair
loops. They expose imports, symbols, native compiler scopes, symbol-table name
bindings, AST name and attribute references, call sites, decorators,
docstrings, and diagnostics without requiring consumers to parse large JSON
payloads first.
