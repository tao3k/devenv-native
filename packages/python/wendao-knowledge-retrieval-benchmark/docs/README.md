# Wendao Knowledge Retrieval Benchmark Docs

This directory documents the `wendao-knowledge-retrieval-benchmark` package.
The package is the Python-owned black-box judge for Wendao knowledge retrieval
profiles. It reads existing real-repository precision receipts and emits
comparison reports without changing Rust search behavior or Julia service
contracts.

## Contents

- [Architecture](architecture.md): package ownership, data flow, boundaries,
  and non-goals.
- [Profile Contract](profile_contract.md): source receipt expectations, report
  schema, score fields, and extension rules for future profiles.
- [Usage](usage.md): CLI usage, validation commands, and project-harness
  management.

## Stable Package Surfaces

- [Package README](../README.md)
- [Package configuration](../pyproject.toml)
- [CLI entrypoint](../src/wendao_knowledge_retrieval_benchmark/cli.py)
- [Profile scoring](../src/wendao_knowledge_retrieval_benchmark/profiles.py)
- [Package harness tests](../tests/test_project_harness.py)

## Documentation Rules

These docs are canonical package docs. They use repository-relative links and
placeholder paths in command examples. Operational receipts and generated
reports are caller-provided artifacts and are not stable documentation targets.
