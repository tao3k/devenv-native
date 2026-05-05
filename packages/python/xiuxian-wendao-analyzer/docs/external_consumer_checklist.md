# External Consumer Checklist

Use this checklist before trying `xiuxian-wendao-analyzer` from another
workspace.

## Environment

Confirm:

1. Python `>=3.12`
2. `pyarrow>=14.0.0`
3. `uv` is available
4. you plan to run the shipped examples with `uv run python ...`

Plain `python examples/...` is not the recommended path for the packaged
examples because the package-local workspace wiring is already encoded in the
`uv` setup.

## Fast Local Validation

Run one shipped example unchanged:

```bash
uv run python examples/scripted_repo_search_workflow.py
uv run python examples/attachment_pdf_analyzer_workflow.py
uv run python examples/document_extraction_workflow.py
```

That proves:

1. the package imports cleanly
2. `WendaoArrowSession.for_repo_search_testing(...)` works in your environment
3. `WendaoArrowSession.attachment_search(...)` works for a scripted PDF attachment workflow
4. `extract_document_table(...)` can produce Arrow resource rows through the multi-format document workflow
5. the analyzer package can process scripted Rust-shaped repo-search rows

## Optional Document Parsing

Real Docling conversion is optional:

```bash
uv sync --extra documents
uv run python examples/document_extraction_workflow.py --mode docling --source path/to/document.docx
```

The `documents` extra includes Docling's XBRL dependency. Use
`documents-audio` when running real audio ASR fixtures:

```bash
uv sync --extra documents-audio
```

Use `extract_document_table(...)` when you need an Arrow table and
`extract_document_resources(...)` when you need typed Python rows first.
Docling is the parser authority for actual support. The analyzer exports
`DOCLING_SUPPORTED_DOCUMENT_FORMATS`, `DOCLING_COMMON_SOURCE_SUFFIXES`, and
`is_known_docling_source(...)` for UI and preflight hints.
Reusable extraction cache rows are stored as Arrow IPC `_resources.arrow`;
JSON is not part of the Python-to-Rust extraction contract.
The `docling_json` resource row points at the exported JSON file and leaves
`content` empty to keep cache-hit Arrow payloads small.
The stable Arrow schema is shared across all extracted resource rows. Docling
may produce a main markdown `document` row plus structured `table`, `image`,
`formula`, `code`, `docling_json`, `audio`, and `subtitle` rows.

For Wendao integration, start the document extraction service:

```bash
uv run wendao-document-extract --host 0.0.0.0 --port 50051
```

The Flight route is `/analysis/document-extract`.
The Python service is the synchronous Arrow Flight worker. Rust Wendao
providers own async queueing, content-hash deduplication, DuckDB job registry
state, and browser-facing status endpoints.

The Python service to Rust provider performance path is covered by an ignored
Cargo test and the local benchmark driver:

```bash
uv run python tests/scripts/benchmark_wendao_document_extract.py
```

The default benchmark uses fake converter fixtures, including audio and image
inputs, so it remains deterministic. Real Docling benchmarking is opt-in and
can prepare a sparse local fixture checkout with Docling's own `tests/data`
attachments:

```bash
uv sync --extra documents
uv run python tests/scripts/benchmark_wendao_document_extract.py --prepare-only
uv run python tests/scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --prepare-docling-fixtures \
  --python-uv-extra documents \
  --fail-on-error-rows
```

For Rust-owned async queue and dedup validation, keep Python as the synchronous
worker and let the benchmark driver start the existing Rust Flight provider:

```bash
uv run python tests/scripts/benchmark_wendao_document_extract.py \
  --flight-mode async \
  --wait-ms 5000 \
  --duplicate-miss-concurrency 20 \
  --fail-on-duplicate-conversions \
  --fail-on-error-rows
```

Local benchmark runs isolate the OCR shard cache by default. Pass
`--ocr-shard-cache-root` only when the run should intentionally reuse a
persistent shard cache. Use `--rust-pdf-ocr-source-range-workers` only for
profiling source-PDF page-range chunk counts; production defaults should stay
adaptive unless benchmark evidence justifies an override.

When a Rust gateway REST endpoint is already available, add:

```bash
--rust-rest-endpoint http://127.0.0.1:<gateway-port>
```

This samples `GET /api/document-extract-jobs` during each benchmark probe and
records queue depth, running job count, scheduled in-process jobs, conversion
capacity, permit pressure, and last/max conversion duration in the JSON and
Markdown reports.
Use `--distinct-miss-concurrency <n>` for a separate capacity slice where
different documents cold-miss concurrently through the same Rust provider. The
result adds a `distinctMiss` report section with fixture count, converter call
count, error rows, queue depth, running conversions, permit pressure, and
configured conversion capacity. Keep real Docling duplicate-miss and
distinct-miss checks as separate invocations so each remains a cold-miss
measurement.
For a self-contained local gateway pressure run, add
`--rust-provider-mode gateway`. The benchmark starts a temporary Valkey process,
the synchronous Python worker, and `wendao gateway start`, then samples
`/api/document-extract-jobs` automatically while the Rust probes run.

Use `--skip-audio` when ASR model artifacts should not be loaded. For audio
ASR, install `documents-audio`; the benchmark configures the bundled
`imageio-ffmpeg` executable before starting Docling's ASR pipeline. Use
`--only-fixture <name>` for targeted real fixture diagnostics and
`--docling-source-root` only when you already have a prepared Docling fixture
checkout. Use `--concurrency` to stress cache-hit requests through the Rust
provider path and `--server-start-timeout` when the Docling service has a cold
start. Keep `--fail-on-error-rows` enabled for real fixture runs so conversion
errors cannot pass as successful Arrow rows.
For production-like risk checks, compare total Arrow IPC bytes and cache-hit
p95 before and after changes; first-time PDF/OCR/audio conversion should be
planned as Rust-owned asynchronous worker capacity rather than Python-side
request-thread registry work.
Full-profile Docling conversions are subprocess-isolated by default. This keeps
native crashes in heavyweight Docling, Torch, or model-runtime paths scoped to a
single extraction instead of killing the Arrow Flight worker. Keep
`WENDAO_DOCUMENT_EXTRACT_FULL_ISOLATION` enabled for service runs; use
`WENDAO_DOCUMENT_EXTRACT_FULL_ISOLATION=false` only for local inline debugging.
Use `WENDAO_DOCUMENT_EXTRACT_FULL_TIMEOUT_SECONDS` to lower or raise the
default 900 second child-process timeout. Attachment-oriented `fast-text`
requests are not subprocess-isolated because they avoid the heavyweight
OCR/table-structure profile and should stay on the low-latency path.
Use `WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS` to cap simultaneous
Rust-dispatched cold conversions. The default follows host parallelism and is
bounded to four conversions, while queued jobs stay visible through the
document extraction status route. `GET /api/document-extract-jobs` returns the
runtime snapshot for this pool and the DuckDB-backed job registry, including
queued/running/succeeded/failed counts, scheduled in-process jobs, conversion
capacity, and last/max conversion duration. `GET
/api/document-extract-job?job_id=...` remains the single-job status route.

## Host-Backed Repo Search

If you want real Wendao search results, confirm these binaries exist:

```bash
cargo build -p xiuxian-wendao --features julia --bin wendao_search_flight_server --bin wendao_search_seed_sample
```

Then you can seed a temporary workspace and run:

```bash
tmp_root="$(mktemp -d)"
wendao_search_seed_sample alpha/repo "$tmp_root"
uv run python examples/repo_search_workflow.py --host 127.0.0.1 --port 8815
uv run python examples/custom_repo_analyzer_workflow.py --host 127.0.0.1 --port 8815
uv run python examples/host_backed_repo_search_beta_smoke.py --port 0
uv run python examples/host_backed_repo_search_beta_smoke.py --mode custom --port 0
uv run python examples/host_backed_repo_search_beta_smoke.py --port 0 --keep-workspace
```

Use `--keep-workspace` when you want to inspect the seeded repo after the smoke
run.

## Generic Table Or Row Analysis

If another package already fetched a Rust-owned query result, you do not need a
repo-search-specific entrypoint.

Use:

1. `analyze_rows(...)`
2. `analyze_table(...)`
3. `run_rows_analysis(...)`
4. `run_table_analysis(...)`

## Rerank Boundary

If your workflow needs rerank data:

1. fetch it through `wendao-core-lib` or `wendao-arrow-interface`
2. keep rerank transport ownership there
3. hand the returned table into `analyze_table(...)` if you want Python-side
   post-analysis

There is no analyzer-owned rerank workflow in this package's beta contract.
