# Write Your First Custom Analyzer

## The Smallest Honest Contract

A custom analyzer only needs one method:

```python
class CustomAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ...
```

Your returned rows should include a stable `rank` field.

## A Minimal Example

```python
class CustomAnalyzer:
    def analyze_rows(self, rows: list[dict[str, object]]) -> list[dict[str, object]]:
        ranked = sorted(rows, key=lambda row: float(row["score"]), reverse=True)
        return [
            {
                "path": str(row["path"]),
                "score": float(row["score"]),
                "rank": index + 1,
            }
            for index, row in enumerate(ranked)
        ]
```

## Fastest Local Authoring Loop

Prefer the shared scripted testing surface while authoring:

1. `WendaoArrowSession.for_repo_search_testing(...)`
2. `run_repo_analysis(...)`
3. `summarize_repo_analysis(...)`

The shipped reference workflow is
[`custom_repo_analyzer_workflow.py`](../examples/custom_repo_analyzer_workflow.py),
and the offline companion is
[`scripted_repo_search_workflow.py`](../examples/scripted_repo_search_workflow.py).
For a non-repo example with a Rust-owned attachment route, also see
[`attachment_pdf_analyzer_workflow.py`](../examples/attachment_pdf_analyzer_workflow.py).
For local document parsing into Arrow rows, see
[`document_extraction_workflow.py`](../examples/document_extraction_workflow.py)
and `extract_document_table(...)`.

## Analyze Already Materialized Data

If transport happened somewhere else, keep the same analyzer object and switch
to the lower-level runtime helpers:

1. `analyze_rows(...)`
2. `analyze_table(...)`
3. `run_rows_analysis(...)`
4. `run_table_analysis(...)`

This is the intended bridge for Rust-owned routes that are not repo-search.
The shipped PDF attachment example uses the same rule:
`WendaoArrowSession.attachment_search(...)` materializes the table first, then
`run_table_analysis(...)` applies Python logic over that returned data.

For Docling-supported document parsing, `extract_document_table(...)` returns
the Arrow-shaped resource table directly. Use the optional `documents` extra
when running the real Docling converter. Use
`DOCLING_SUPPORTED_DOCUMENT_FORMATS`, `DOCLING_COMMON_SOURCE_SUFFIXES`, and
`is_known_docling_source(...)` only for UI or preflight hints; the converter is
still the parser authority.

## Boundary Rule

`xiuxian-wendao-analyzer` owns the workflow seam between returned data and
Python analysis logic. It does not own rerank transport or a Python-side rerank
runtime.

If you need `/rerank/flight`, fetch it through `wendao-core-lib` or
`wendao-arrow-interface`, then analyze the returned table with
`analyze_table(...)`.
