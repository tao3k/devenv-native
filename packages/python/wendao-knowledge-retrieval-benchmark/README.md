# Wendao Knowledge Retrieval Benchmark

`wendao-knowledge-retrieval-benchmark` is the Python-owned black-box benchmark
harness for Wendao knowledge retrieval profiles.

The package reads existing `xiuxian_wendao.real_repo_search_precision.v1`
receipts and compares retrieval strategies without changing Rust runtime
routes, Julia services, Arrow schemas, or search-ranking behavior.

The first supported profiles are:

- `flat-topk`: estimates the cost of exposing all observed top-k result paths
  from linked query receipts.
- `graph-first-reasoning-tree`: estimates progressive disclosure cost from
  scenario reasoning-tree receipts, including anchors, semantic relation hops,
  PageIndex seed evidence, source evidence, and disclosure depth.

Future slices can add optional Julia-backed PPR, community-frontier, and hybrid
profiles as measured backends. Python remains the benchmark judge; Rust and
Julia remain benchmarked implementations.

The package is managed by `python-lang-project-harness` through its local
project configuration and package-level harness test.

## Documentation

- [Package docs](docs/README.md)
- [Architecture](docs/architecture.md)
- [Profile contract](docs/profile_contract.md)
- [Usage](docs/usage.md)

## Usage

```bash
wendao-knowledge-retrieval-benchmark \
  --receipt path/to/real_repo_receipt.json \
  --output-json path/to/knowledge_retrieval_benchmark.json \
  --output-markdown path/to/knowledge_retrieval_benchmark.md
```
