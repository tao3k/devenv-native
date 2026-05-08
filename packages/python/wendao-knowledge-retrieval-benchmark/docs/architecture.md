# Architecture

`wendao-knowledge-retrieval-benchmark` is a Python package for black-box
retrieval profile comparison. It does not own Wendao indexing, ranking,
runtime routes, Julia execution, or Arrow schemas. Its job is to evaluate
evidence that those systems already produced.

## Ownership

The package owns:

- loading `xiuxian_wendao.real_repo_search_precision.v1` receipts;
- computing comparable profile rows from those receipts;
- rendering JSON and Markdown benchmark reports;
- package-level validation through `python-lang-project-harness`.

Rust owns:

- real-repository catalog and precision receipt generation;
- LinkGraph indexing and search ranking;
- semantic/PageIndex evidence extraction before any optional Julia dispatch.

Julia owns future measured implementations such as PPR-like relatedness,
community-frontier exploration, and large graph traversal. This package may
score those implementations only after their outputs appear as explicit
profile inputs.

## Data Flow

1. A Wendao precision run emits a real-repository receipt.
2. The benchmark CLI reads that receipt from a caller-provided path.
3. Profile scorers compute comparable `ProfileScore` rows.
4. The report renderer writes JSON and Markdown to caller-provided paths, or
   prints Markdown to stdout.

The package is deliberately offline and deterministic once the receipt exists.
It does not refresh repositories, start services, call Julia, or query a live
search endpoint.

## Current Profiles

`flat-topk` estimates the cost of exposing all observed top-k paths linked to
the scenario queries.

`graph-first-reasoning-tree` estimates progressive disclosure over scenario
reasoning-tree steps: anchor query, semantic relation, PageIndex seed, and
source evidence.

The first real comparison keeps quality equal while reducing exposed
path-character cost:

| Profile                      | Scenarios | Recall@10 |  MRR | Exposed chars | Steps | Max depth |
| ---------------------------- | --------: | --------: | ---: | ------------: | ----: | --------: |
| `flat-topk`                  |       7/7 |     10000 | 9285 |         13777 |     0 |         0 |
| `graph-first-reasoning-tree` |       7/7 |     10000 | 9285 |          4101 |    31 |         2 |

## Non-Goals

This package does not:

- implement a search algorithm;
- change Rust or Julia runtime behavior;
- replace the Rust precision harness;
- treat speculative Julia profiles as accepted evidence;
- make hidden operational paths part of canonical docs.
