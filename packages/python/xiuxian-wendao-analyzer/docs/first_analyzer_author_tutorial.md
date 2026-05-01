# First Analyzer Author Tutorial

`xiuxian-wendao-analyzer` sits above the Rust-owned Wendao transport stack:

1. `wendao-core-lib` owns Flight transport and typed contracts
2. `wendao-arrow-interface` owns downstream session ergonomics
3. `xiuxian-wendao-analyzer` owns analysis over rows and tables that already
   came back from Rust
4. `xiuxian-wendao-analyzer` owns the Wendao-facing document extraction service
   adapter for Docling-backed parsing

The important boundary is simple: this package does not own rerank workflows.
If Rust returns a table, this package can analyze that table. It does not
define a separate Python-side rerank runtime.

## Workflow 1: Offline Repo Search Authoring With Scripted Results

Start here when you want the fastest local loop and do not need a live Flight
host.

```bash
uv run python examples/scripted_repo_search_workflow.py
```

That workflow uses:

1. `WendaoArrowSession.for_repo_search_testing(...)`
2. `run_repo_analysis(...)`
3. `summarize_repo_analysis(...)`

## Workflow 2: Host-Backed Repo Search With Built-In Ranking

Use this when you want real Wendao repo-search data and the built-in
`score_rank` analyzer is enough.

```bash
uv run python examples/repo_search_workflow.py --help
```

The built-in path is:

1. `run_repo_analysis(...)`
2. `summarize_repo_analysis(...)`
3. `AnalyzerConfig(strategy="score_rank")`

## Workflow 3: Host-Backed Repo Search With A Custom Python Analyzer

Use this when Rust should fetch the rows but your ranking logic is custom.

```bash
uv run python examples/custom_repo_analyzer_workflow.py --help
```

That workflow keeps ownership clean:

1. Rust fetches the data
2. your analyzer object implements `analyze_rows(...)`
3. `run_repo_analysis(...)` applies it to the returned rows

## Workflow 4: PDF Attachment Search Then Analyze The Returned Table

Use this when Rust should query `/search/attachments` and your Python analyzer
should only work over the returned PDF rows.

```bash
uv run python examples/attachment_pdf_analyzer_workflow.py
```

That workflow keeps the boundary explicit:

1. `attachment_search_request(...)` builds the Rust-owned query contract
2. `WendaoArrowSession.attachment_search(...)` fetches the Arrow table
3. `run_table_analysis(...)` analyzes the returned table in Python

If you already have a live Flight endpoint that serves `/search/attachments`,
switch the same example to endpoint mode with `--mode endpoint --port <port>`.

## Workflow 5: Docling Document Extraction Into Arrow Rows

Use this when Python should parse a local Docling-supported source and produce
Arrow-shaped resource rows. The documented Docling input set includes PDF,
DOCX, XLSX, PPTX, Markdown, AsciiDoc, HTML/XHTML, CSV, image formats such as
PNG/JPEG/TIFF/BMP/WEBP, XML-based patent or article formats, XBRL XML,
METS GBS, WebVTT, LaTeX, plain text, audio, and Docling JSON.

```bash
uv run python examples/document_extraction_workflow.py
```

Install the optional document parser dependency before using real Docling mode:

```bash
uv sync --extra documents
uv run python examples/document_extraction_workflow.py --mode docling --source path/to/document.docx
```

The helper surface is:

1. `extract_document_table(...)`
2. `extract_document_resources(...)`
3. `DOCLING_SUPPORTED_DOCUMENT_FORMATS` and `DOCLING_COMMON_SOURCE_SUFFIXES`
   for downstream UX
4. `is_known_docling_source(...)` as a suffix helper, not a parser gate
5. `extract_pdf_table(...)` for PDF compatibility callers

The stable Arrow resource schema covers all extracted rows. The helper always
emits a main markdown `document` row and may also emit Docling-backed
structured rows such as `table`, `image`, `formula`, `code`, `docling_json`,
`audio`, and `subtitle`.

For Wendao integration, run the Arrow Flight service:

```bash
uv run wendao-document-extract --host 0.0.0.0 --port 50051
```

The route is `/analysis/document-extract`.

## Workflow 6: Analyze An Already Materialized Rust Query Result

If another package already fetched the data, analyze it directly.

```python
from wendao_arrow_interface import WendaoArrowSession
from xiuxian_wendao_analyzer import analyze_table

session = WendaoArrowSession.for_repo_search_testing(
    [{"path": "src/lib.rs", "score": 0.9}]
)
result = session.repo_search("alpha", limit=1)
ranked = analyze_table(result.table)
```

The same pattern applies to any Rust-owned route. For example, if you later
fetch `/rerank/flight` through `wendao-arrow-interface`, hand the returned table
to `analyze_table(...)` instead of looking for a rerank-specific analyzer API.
