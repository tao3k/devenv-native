# Usage

The package exposes one CLI:

```bash
wendao-knowledge-retrieval-benchmark \
  --receipt path/to/real_repo_receipt.json \
  --output-json path/to/knowledge_retrieval_benchmark.json \
  --output-markdown path/to/knowledge_retrieval_benchmark.md
```

If no output path is provided, the CLI writes the Markdown report to stdout:

```bash
wendao-knowledge-retrieval-benchmark \
  --receipt path/to/real_repo_receipt.json
```

The receipt must use the
`xiuxian_wendao.real_repo_search_precision.v1` schema.

## Validation

Run package checks from the repository root:

```bash
direnv exec . uv run ruff check packages/python/wendao-knowledge-retrieval-benchmark
direnv exec . uv run ruff format --check packages/python/wendao-knowledge-retrieval-benchmark
direnv exec . uv run pytest packages/python/wendao-knowledge-retrieval-benchmark/tests -q
```

The package also has an explicit project-harness gate:

```bash
direnv exec . uv run pytest packages/python/wendao-knowledge-retrieval-benchmark/tests/test_project_harness.py -q
```

## Project Harness Management

`pyproject.toml` configures `python-lang-project-harness` with error-level
blocking. The package-level harness test keeps the benchmark package aligned
with the Python project-harness policy while leaving advisory findings
non-blocking.

When adding a new profile scorer, update:

1. [Profile scoring](../src/wendao_knowledge_retrieval_benchmark/profiles.py)
2. [Profile contract docs](profile_contract.md)
3. [Package tests](../tests/test_benchmark.py)
4. [Package README](../README.md) if the public profile list changes
