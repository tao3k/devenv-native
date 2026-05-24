---
type: knowledge
kind: package-readme
title: "xiuxian-wendao-analyzer"
category: "package"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Python analyzer and Docling Flight worker boundary for Wendao document extraction and table analysis."
tags:
  - python
  - analyzer
  - docling
  - arrow-flight
metadata:
  title: "xiuxian-wendao-analyzer"
  retrieval:
    saliency_base: 6.4
    decay_rate: 0.04
---

# xiuxian-wendao-analyzer

`xiuxian-wendao-analyzer` is a beta Python analyzer and document parsing
package built on top of `wendao-core-lib`.

Its boundary is intentionally narrow:

1. analyze rows and Arrow tables that already came back from Rust-owned Wendao
   query or exchange surfaces
2. provide one built-in deterministic ranking strategy for score-carrying rows
3. expose lightweight run objects and summary models for downstream callers
4. reuse `wendao-arrow-interface` for offline scripted authoring and testing
5. provide Docling-backed document extraction helpers that return Arrow-shaped
   resource rows
6. expose a Wendao-facing Arrow Flight service for document extraction

It does not own:

1. rerank transport contracts
2. local rerank semantics
3. Flight metadata assembly
4. any Python-hosted shadow of Rust runtime logic

## Current Beta Surface

The current beta exports:

1. `AnalyzerConfig`
2. `ScoreRankAnalyzer`
3. `AnalyzerResultRow`
4. `AnalysisSummary`
5. `RowsAnalysisRun`
6. `TableAnalysisRun`
7. `QueryAnalysisRun`
8. `RepoAnalysisRun`
9. `build_analyzer(config)`
10. `analyze_rows(...)`
11. `analyze_table(...)`
12. `analyze_query(...)`
13. `analyze_repo_search(...)`
14. `analyze_repo_query_text(...)`
15. `run_rows_analysis(...)`
16. `run_table_analysis(...)`
17. `run_query_analysis(...)`
18. `run_repo_analysis(...)`
19. `run_repo_search_analysis(...)`
20. `extract_document_table(...)`
21. `extract_document_resources(...)`
22. `DOCLING_SUPPORTED_DOCUMENT_FORMATS`
23. `DOCLING_COMMON_SOURCE_SUFFIXES`
24. `is_known_docling_source(...)`
25. `extract_pdf_table(...)` and `extract_pdf_resources(...)` compatibility wrappers
26. `DocumentExtractFlightServer`
27. `build_document_extract_table(...)`
28. `/analysis/document-extract` as the primary document extraction route
29. `DoclingPdfOcrShardWorker` for opt-in page-shard OCR
30. `/analysis/audio-shards` as the internal Flight/Arrow route for Rust-owned
    audio shard processing
31. `wendao-image-ocr-jsonl` and `wendao-docling-document-jsonl` as
    queue-keyed source-contract evidence adapters
32. summary helpers over the same rows, table, query, and repo-search runs

Docling is optional through the `documents` extra. That extra includes
Docling's XBRL support so the documented XML/XBRL coverage is real, not only a
suffix hint:

```bash
uv sync --extra documents
```

Audio ASR needs heavier model and media dependencies. Use the dedicated audio
extra when running real audio conversion:

```bash
uv sync --extra documents-audio
```

For bounded MP3 audio diagnostics, use the package-managed test script instead
of adding a public command surface:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py <recording-dir> \
  --backend both \
  --openrouter-base-url https://openrouter.ai/api/v1/audio/transcriptions \
  --openrouter-model qwen/qwen3-asr-flash-2026-02-10 \
  --local-language zh \
  --sample-strategy uniform \
  --audio-materialization-mode normalized-16k-wav \
  --hosted-request-concurrency 4 \
  --limit-files 2 \
  --limit-chunks 1
```

Production chunk planning belongs on the Rust side in
`xiuxian-wendao-attachments::audio`: it derives logical chunk offsets, optional
context windows, normalized media windows, shard cache keys, and downstream
task/backend result cache keys before Python sees a backend request. The same
Rust boundary can materialize normalized shard media in parallel with local
`ffmpeg`, so Gateway/Studio can avoid Python-owned chunking on the hot path.
The Python diagnostic mirrors that contract only for package tests and local
comparison runs. It writes the backend-neutral
`audio_shards.json` manifest, then compares Docling, hosted audio-input models,
and explicit local OpenAI-compatible candidates on the same chunks. OpenRouter
runs require the standard
`OPENROUTER_API_KEY` environment variable. The script writes JSON summaries and
transcript files under the selected output directory. It also writes
`quality.json` and `review.tsv` with proxy precision signals such as empty
outputs, Chinese character ratio, inaudible-marker density, characters per
minute, and optional character error rate when a reference JSONL transcript is
provided. Diagnostic inputs default to `--input-privacy private-local`, which
requires output under `.cache/agent/evidence`; use
`--allow-private-output-outside-cache` only for local scratch directories that
will not be committed. Use `--domain-terms-file` to append a private glossary
to hosted prompts, and `--required-terms-file` with
`--min-required-term-recall` to mark critical term loss in `quality.json` and
`review.tsv`. The shard manifest uses `xiuxian_wendao.audio_shards.v1` and
`audio-shards-v1`; those names are model-neutral so local and hosted backends
can change without changing chunk/cache identity.

Audio materialization is explicit in diagnostics because the private evaluation
set is MP3. The default `normalized-16k-wav` decodes bounded shards to 16 kHz
mono WAV for broad model compatibility. `native-rate-wav` still decodes
bounded shards, but preserves the source sample rate while using mono WAV.
`source-direct` sends the original full source file, such as MP3, without
ffmpeg chunking; use it only for backend compatibility and precision
diagnostics because it cannot represent sub-source shard windows without
decoding or trimming.

For VAD-guided diagnostics, pass `--sample-strategy speech-segments` with
`--speech-segments-jsonl <segments.jsonl>`. The segment sidecar is intentionally
model-neutral: each JSONL row may include `source` or `sourceId`,
`startSeconds` or `startMs`, and either `durationSeconds`/`durationMs` or
`endSeconds`/`endMs`. This lets local CoreML VAD experiments, Rust-side audio
materialization, or any future hosted speech detector provide precise speech
windows without coupling the analyzer to one ASR model. The resulting shard
manifest still uses the same `xiuxian_wendao.audio_shards.v1` identity surface.
The production Rust document-extract route can consume the same sidecar through
`WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL` and use it only for
failed-row recovery planning. Python still receives unchanged audio shard Arrow
rows and remains the backend invocation adapter.

The document extraction Flight service uses the same model-neutral audio
contract for real backend calls. Start the service with `--audio-worker skip`,
`--audio-worker docling`, or `--audio-worker hosted`; the stable backend
profiles are `docling-audio-transcript-v1` and
`hosted-audio-transcript-v1`. The managed Wendao analyzer service passes
`--audio-worker hosted` by default and selects OpenRouter unless explicitly
configured for a local OpenAI-compatible backend. `--audio-workers` and the
`x-wendao-audio-workers` Flight metadata header bound analyzer-side request
parallelism inside the Rust-owned shard budget. Hosted audio uses
`WENDAO_AUDIO_HOSTED_PROVIDER`, `WENDAO_AUDIO_HOSTED_BASE_URL`,
`WENDAO_AUDIO_HOSTED_MODEL`, `WENDAO_AUDIO_HOSTED_API_KEY`,
`WENDAO_AUDIO_HOSTED_TIMEOUT_SECONDS`, and
`WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY`; when the provider is `openrouter`,
`OPENROUTER_API_KEY` is accepted as the public key fallback. The analyzer
returns failed rows for missing configuration, malformed hosted responses,
empty transcript text, backend request errors, and unsafe transcript content so
Rust can keep precision and coverage gates deterministic. The hosted worker's
content gate is model-neutral: it rejects excessive characters per minute,
high repeated character n-gram ratio, hosted refusal text,
no-transcribable-speech meta responses, and high Latin text ratio when the
shard is configured for Chinese. The gate is enabled by default and can be
tuned with `WENDAO_AUDIO_TRANSCRIPT_QUALITY_GATE`,
`WENDAO_AUDIO_TRANSCRIPT_MAX_CHARS_PER_MINUTE`,
`WENDAO_AUDIO_TRANSCRIPT_MAX_REPEATED_NGRAM_RATIO`, and
`WENDAO_AUDIO_TRANSCRIPT_MAX_LATIN_RATIO_FOR_CHINESE`.

Local model automation uses a shared analyzer `local_backend` substrate for
device probes, launch descriptors, environment resolution, project cache/data
roots, module-local adapter paths, and long-running backend process execution.
OCR2 and audio keep separate backend packages and runner policy, so audio does
not share OCR-specific code and OCR2 does not depend on audio implementation
details. Use
`--audio-probe-local-backend` to inspect the platform-selected runner, and
`--audio-start-backend` to start a local OpenAI-compatible audio endpoint. On
macOS Apple Silicon, `auto` selects the `qwen3-asr-mlx` runner and serves the
same chat/audio `input_audio` shape used by the hosted worker. FireRedASR2S is
not a Metal runner; its upstream CLI exposes CUDA-style `.cuda()` acceleration,
so the setup helper refuses CPU fallback and reports MPS/Metal as blocked for
that runner. `qwen3-asr-mlx` is an explicit Apple Silicon candidate through
`mlx-qwen3-asr`; it is useful for local Chinese ASR
experiments, but it is not promoted by default and must pass the same transcript
truth/CER and repetition gates as every other backend.
`local-openai-audio` is only the invocation channel for a local
OpenAI-compatible audio endpoint; diagnostics and Org timelines record the
actual model separately through `model` / `:MODEL:`.

Promotion is gated by a curated Chinese transcript truth set, not only by
agreement between Docling and hosted models. Reports should keep character
error rate, critical number/entity preservation, shard coverage, duplicate
span checks, and backend latency separate from Rust shard materialization and
merge time. Private recordings may be used for local diagnostics only; any
committed truth fixtures must be separately approved, redacted, or synthesized.
The diagnostic writes `truth_template.jsonl`, `reference_draft.jsonl`, and
`reference_draft.tsv`. The truth template stays blank. The reference draft is
prefilled from candidate transcripts and is marked
`referenceStatus: candidate-draft`, so it is rejected by the precision gate
until reviewed. After correcting the draft text, convert it into a
promotion-safe reference file. Every row must be explicitly marked
`referenceStatus: curated` before conversion; the converter rejects
`candidate-draft` rows so model-generated drafts cannot become truth by
accident:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py \
  --curate-reference-draft <edited-reference-draft.jsonl> \
  --curated-reference-jsonl <curated-reference.jsonl>
```

or:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py \
  --curate-reference-tsv <edited-reference-draft.tsv> \
  --curated-reference-jsonl <curated-reference.jsonl>
```

Then pass `<curated-reference.jsonl>` back through `--reference-jsonl` for CER
and critical-term scoring. Before running ASR, validate the curated reference
against the diagnostic shard manifest:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py \
  --validate-reference-jsonl <curated-reference.jsonl> \
  --reference-audio-shards-json <audio-shards.json> \
  --reference-validation-report-json <reference-validation-report.json>
```

For audio, the shard timeline is the structure authority. The validator checks
that the reference rows cover the manifest and that the manifest timeline is
valid: shard ids and reading-order keys must be unique, shard starts must be
monotonic, durations must be positive, and the materialized media window must
cover the logical shard window.

Minimal reference rows without candidate backend metadata are also accepted.
For the current Chinese PI private-audio lane, the next precision rerun is
limited to the local `Qwen/Qwen3-ASR-1.7B` MLX endpoint and OpenRouter
`qwen/qwen3-asr-flash-2026-02-10` through the speech-to-text
`/audio/transcriptions` endpoint. Gemini and chat/audio Xiaomi requests remain
historical rejected evidence for this lane.
Hosted diagnostics run ordered serial requests unless
`--hosted-request-concurrency` is set explicitly. Production Rust-to-Python
audio shard calls receive their analyzer worker budget from the Rust Flight
metadata selected by the polyglot control plane, so diagnostic concurrency is
not treated as the system admission policy.
After the curated reference validates with `ready=true`, run the local
candidate:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py <recording-file> \
  --backend local-openai-audio \
  --openrouter-base-url http://127.0.0.1:8013/v1/chat/completions \
  --openrouter-model qwen3-asr-1.7b-mlx \
  --limit-files 1 \
  --limit-chunks 5 \
  --chunk-seconds 60 \
  --sample-strategy head \
  --audio-materialization-mode native-rate-wav \
  --reference-jsonl <curated-reference.jsonl>
```

Then run the hosted OpenRouter Qwen ASR Flash comparator:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py <recording-file> \
  --backend openrouter-audio \
  --openrouter-base-url https://openrouter.ai/api/v1/audio/transcriptions \
  --openrouter-model qwen/qwen3-asr-flash-2026-02-10 \
  --limit-files 1 \
  --limit-chunks 5 \
  --chunk-seconds 60 \
  --sample-strategy head \
  --audio-materialization-mode native-rate-wav \
  --reference-jsonl <curated-reference.jsonl>
```

Compare candidate summaries with precision as the hard gate before wall time:

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py \
  --compare-summary-json <qwen-summary.json> <xiaomi-summary.json> \
  --comparison-report-json <comparison-report.json>
```

For a Chinese-first local ASR candidate, provision FireRedASR2S as an isolated
diagnostic tool, then pass the emitted command into the shared diagnostic:

```bash
direnv exec . uv run python tests/scripts/fireredasr2s_local_setup.py \
  --download-models \
  --summary-json <setup-summary-json>
```

```bash
direnv exec . uv run python tests/scripts/audio_asr_diagnostic.py <recording-dir> \
  --backend firered-openrouter \
  --fireredasr2s-command "<fireRedAsr2sCommand from setup summary>" \
  --openrouter-base-url https://openrouter.ai/api/v1/audio/transcriptions \
  --openrouter-model qwen/qwen3-asr-flash-2026-02-10 \
  --sample-strategy uniform \
  --limit-files 2 \
  --limit-chunks 1
```

The FireRedASR2S adapter calls its local CLI on the same normalized 16 kHz mono
chunks used by OpenRouter, then feeds the result into the same quality review
files. The setup helper pins the official source revision, creates a separate
virtual environment, downloads the AED/VAD/LID/punctuation weights under the
project model directory, and refuses CPU fallback. FireRedASR2S remains an
environment-provided diagnostic backend, not a required analyzer dependency.
Its upstream CLI accelerates through CUDA-style `.cuda()` calls; on macOS
Metal/MPS hosts it is blocked until the upstream runner gains a model-neutral
device path or FireRedASR2S is exposed through a separate hosted
OpenAI-compatible service.

For local Qwen3-ASR MLX experiments on Apple Silicon:

```bash
direnv exec . uv run wendao-document-extract \
  --audio-start-backend \
  --audio-backend-runner qwen3-asr-mlx \
  --audio-backend-model-path Qwen/Qwen3-ASR-1.7B \
  --audio-backend-host 127.0.0.1 \
  --audio-backend-port 8013
```

Then point `audio_asr_diagnostic.py --backend local-openai-audio` at
`http://127.0.0.1:8013/v1/chat/completions`; keep `local-openai-audio` as the
channel label and use the configured Qwen3-ASR model id as the model label. A
May 15, 2026 private five-minute diagnostic kept `Qwen/Qwen3-ASR-1.7B` as the
current local Mandarin precision candidate by proxy: zero failed rows, zero
weak rows, stable Chinese output, and low repetition after warmup. It is still
not promoted until a curated reference transcript supplies CER and critical
entity/number checks. Do not use `mlx-community/*-8bit` weights with this
adapter; the current runner expects the `mlx-qwen3-asr` model layout.
The adapter serves `qwen3-asr-1.7b-mlx` by default and requests timestamp
chunks by default so VTT/SRT/Org review outputs can use model-provided segment
times when the local runner returns them.

Docling is the parsing authority. The analyzer does not maintain a runtime
allowlist; it exposes known common Docling formats and suffixes for downstream
UX. The current documented set includes PDF, DOCX, XLSX, PPTX, Markdown,
AsciiDoc, HTML/XHTML, CSV, PNG, JPEG, TIFF, BMP, WEBP, USPTO XML, JATS XML,
XBRL XML, METS GBS, WebVTT, LaTeX, plain text, audio, and Docling JSON.

## Wendao Document Service

The package is a real Wendao integration package, not only an example bundle.
The document service entrypoints are:

```bash
uv run wendao-document-extract --host 0.0.0.0 --port 50051
uv run xiuxian-wendao-document-extract --host 0.0.0.0 --port 50051
uv run wendao-document-extract --host 0.0.0.0 --port 50051 --pdf-ocr-worker docling
uv run wendao-document-extract --host 0.0.0.0 --port 50051 --pdf-ocr-worker docling --pdf-ocr-workers auto
```

The Arrow Flight route is `/analysis/document-extract`. Request metadata uses:

1. `x-wendao-schema-version`
2. `x-wendao-document-extract-source-path`
3. `x-wendao-document-extract-output-dir`
4. `x-wendao-document-extract-force`
5. `x-wendao-document-extract-error-row`

The returned Arrow table uses the stable document resource schema:

1. `sourcePath`
2. `resourceType`
3. `resourcePath`
4. `pageIndex`
5. `caption`
6. `content`
7. `mimeType`
8. `status`
9. `elementId`

Each extraction emits a main markdown `document` row and may emit structured
rows when Docling exposes reusable content, including `table`, `image`,
`formula`, `code`, `docling_json`, `audio`, and `subtitle`. Reusable rows are
cached in Arrow IPC as `_resources.arrow`; JSON is only an optional exported
resource row and is not the Python-to-Rust transport contract. The
`docling_json` row points at the exported JSON file without inlining that
payload into the Arrow `content` column, so large XML/PDF conversions do not
force every cache-hit response to carry the full Docling JSON export.

The same service also exposes an internal OCR shard exchange route at
`/analysis/pdf-ocr-shards`. This route uses Arrow Flight `do_exchange`, not
JSON metadata: callers upload `xiuxian_wendao.pdf_ocr_shard_input.v1` Arrow
batches and receive `xiuxian_wendao.pdf_ocr_shard_result.v1` Arrow batches.
It is a worker contract for Rust-rendered page or region shards and does not
change the primary `/analysis/document-extract` sync or async extraction path.
Direct CLI invocations without an explicit worker return `skipped` rows so
tests and unmanaged deployments do not load OCR models by accident. The managed
Wendao analyzer service passes `--pdf-ocr-worker docling` by default, enabling
the Docling image worker for rendered shards; failed or empty shard OCR rows
remain table-shaped failures so the Rust hybrid provider can fall back to full
Docling when coverage is incomplete.

For source-contract image evidence tasks that are not PDF page shards, the
package also exposes `wendao-image-ocr-jsonl`. It reads a Rust-written
`tasks.tsv`, sends only `image_ocr_evidence` rows to the configured
OpenAI-compatible Hosted VLM/OCR endpoint, and writes queue-keyed OCR JSONL
rows for downstream cache bridges. Task paths are resolved relative to the
configured corpus root and path escapes are rejected before any source read or
network request. This is an analyzer-side adapter, not a public Gateway route
and not an ontology promotion path. Downstream private episteme runners must
still enforce review-required and no-RDF-promotion semantics before accepting
the text as cache evidence.

For source-contract document evidence tasks, the package exposes
`wendao-docling-document-jsonl`. It reads the same Rust-written `tasks.tsv`,
selects only `document_text_evidence` rows with Docling-supported modern
document extensions in this slice (`pdf`, `docx`, `pptx`, and `xlsx`), runs the
configured Docling profile, and writes queue-keyed Markdown JSONL rows for
downstream private cache bridges. Legacy binary Office inputs such as `.doc`,
`.ppt`, and `.xls` are intentionally skipped by this adapter until a separate
conversion contract produces a supported source. Task paths are confined to
the configured corpus root before Docling is invoked. The adapter is not a
public Gateway route, does not mutate ontology state, and does not make
extracted text eligible for RDF promotion by itself.

These source-contract adapters are intentionally package-owned commands. A
private episteme repository should not carry customer-local Python tools for
the same bridge. The recommended operator flow is:

1. Let Studio write the route-specific `tasks.tsv`.
2. Run `wendao-image-ocr-jsonl` or `wendao-docling-document-jsonl` from this
   analyzer package to produce queue-keyed JSONL.
3. Run the Studio source-contract cache command with `--use-existing-results`
   and the matching result JSONL path.

This split keeps Python responsible for external OCR/Docling connectivity and
keeps Rust responsible for source-contract planning, cache identity, JSONL
validation, and no-RDF-promotion enforcement.

Docling shard OCR is bounded and adaptive. The service accepts
`--pdf-ocr-workers auto|N` as a direct local default, but the Rust provider may
override it per exchange through the internal `x-wendao-pdf-ocr-workers` Flight
metadata header. Rust owns the global OCR worker budget, splits shard batches
into scheduled chunks, and sends only the acquired worker count to Python for
that exchange. Python remains the Docling OCR execution boundary. Full-page PDF
shards from the same source are selected by Rust, grouped into Docling
`sourcePath` page ranges, and then split back into one OCR result row per page.
The full-page source-range selector avoids `pdf-inspector`, PDFium, and
high-DPI rendering on the hot OCR path: Rust reads the PDF page tree with
`lopdf`, emits stable whole-page shard rows, and lets Docling convert the
original `sourcePath` page range. Python first asks Docling for a single
page-break-separated Markdown export for the converted range; if that shape is
unavailable or incomplete, it falls back to Docling's `page_no`-scoped Markdown
export for each page. Rendered images remain the fallback path for failed page
ranges, region shards, and explicit raster tests.
Result rows are still returned in input shard order. The Rust provider also
validates and restores result order against the original shard input rows before
merge, so document order is not coupled to Python worker completion order.
For benchmark canaries, Rust can also retry failed or empty non-hosted,
non-backend-text page OCR rows through the existing Hosted VLM/OCR page profile
before allowing the full-document Docling fallback to run. Enable this with
`--rust-pdf-failed-page-recovery hosted-vlm-page` in the benchmark harness or
`WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY=hosted-vlm-page` in the Rust
provider environment. The default remains disabled. Backend-text page failures
and invalid hosted recovery rows still fall through to the existing precision
fallback.
For benchmark readiness only, the worker can pre-initialize selected Docling OCR
profiles and optionally convert one or more real source pages before the worker
listens. `WENDAO_PDF_OCR_PREWARM_PAGE_INDICES` accepts comma-separated
zero-based page indices and supersedes the legacy single
`WENDAO_PDF_OCR_PREWARM_PAGE_INDEX` value. This does not change OCR output
schemas or live routing. The current milestone evidence rejects broad
multi-page prewarm for pages `5,11` because it preserved precision but
regressed force refresh to `18784.875625 ms`.

The same Flight service exposes `/analysis/audio-shards` for Rust-owned audio
chunk scheduling and materialization. Rust is expected to create normalized
audio shards through `xiuxian-wendao-attachments::audio`, then upload
`xiuxian_wendao.audio_shard_input.v1` Arrow batches to Python. Python returns
`xiuxian_wendao.audio_shard_result.v1` rows and does not own chunk planning,
cache identity, or backend scheduling. Direct CLI invocations without an
explicit worker return `skipped` rows until Docling or hosted audio is
configured. The managed Wendao analyzer service runs with the `documents-audio`
extra and passes `--audio-worker hosted` by default, so MP3/WAV-style
attachments use OpenRouter audio unless `WENDAO_AUDIO_WORKER` or per-request
Flight metadata selects a local OpenAI-compatible backend. Docling audio is an
explicit comparator. Successful rows remain `text/plain`; higher-level
transcript formatting is a separate merge or export concern.

Hosted audio workers normalize quoted environment values before building
OpenAI-compatible requests, so `.env` values such as
`OPENROUTER_API_KEY="..."` and `WENDAO_AUDIO_HOSTED_MODEL="..."` are accepted.
The worker also retries transient hosted request or response failures through
`WENDAO_AUDIO_HOSTED_MAX_ATTEMPTS`, which defaults to `2`. If all attempts fail,
the worker still returns failed result rows and Rust precision gates reject
incomplete coverage. The same retry loop also retries hosted responses that
fail the transcript quality gate; if every attempt is still unsafe, the row is
returned as `failed` so Rust recovery can retry only that shard span. Hosted
request concurrency is normally supplied by Rust through the audio worker
budget header derived from the polyglot control-plane schedule. Direct Python
diagnostics may still set `WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY` explicitly.
Transient socket-capacity failures such as address exhaustion use bounded retry
backoff before the next attempt.

The built-in strategy is intentionally small:

1. `score_rank`
2. consumes rows that already carry a numeric `score`
3. emits the same rows with a stable integer `rank`

## Boundary Reading

The package split is:

1. `wendao-core-lib` owns Arrow Flight transport and typed contracts
2. `wendao-arrow-interface` owns downstream session ergonomics and scripted
   fixtures
3. `xiuxian-wendao-analyzer` owns analysis over already materialized results
   and local document-to-Arrow resource shaping

That means rerank stays transport-owned. If you need to analyze a rerank result,
fetch it through `wendao-core-lib` or `wendao-arrow-interface`, then hand the
returned rows or table into `analyze_rows(...)` or `analyze_table(...)`.

## Workflow Selection Guide

| Workflow                                              | Recommended entrypoint                                                                                     | Analyzer ownership                         | Host involvement                       | Validation status |
| ----------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------ | -------------------------------------- | ----------------- |
| Offline repo-search authoring with scripted results   | `WendaoArrowSession.for_repo_search_testing(...)` + `run_repo_analysis(session.client, ...)`               | downstream user code                       | none                                   | local covered     |
| PDF attachment search then analyze the returned table | `attachment_search_request(...)` + `WendaoArrowSession.attachment_search(...)` + `run_table_analysis(...)` | downstream user code                       | scripted by default, endpoint optional | local covered     |
| Wendao document extraction service                    | `wendao-document-extract` + `/analysis/document-extract`                                                   | analyzer package service adapter           | Arrow Flight                           | local covered     |
| Local multi-format document parsing into Arrow rows   | `extract_document_table(...)` or `extract_document_resources(...)` with the optional `documents` extra     | Docling-backed document extraction helpers | none                                   | local covered     |
| Source-contract Docling document evidence sidecar     | `wendao-docling-document-jsonl` over a Rust-written `tasks.tsv`                                            | Docling-backed JSONL adapter               | Rust cache bridge consumes JSONL       | local covered     |

| Repo search with built-in ranking | `run_repo_analysis(...)` + `summarize_repo_analysis(...)` | built-in `score_rank` | real `wendao_search_flight_server` | real-host covered |
| Repo search with a custom Python analyzer | `run_repo_analysis(...)` + `summarize_repo_analysis(...)` + `analyzer=<your analyzer object>` | downstream user code | real `wendao_search_flight_server` | real-host covered |
| Analyze an already materialized Rust query result | `analyze_rows(...)` or `analyze_table(...)` | built-in `score_rank` or downstream user code | depends on who fetched the data | local covered |

## Documentation Set

This README is the intended v1 public docs surface index:

1. [`docs/first_analyzer_author_tutorial.md`](docs/first_analyzer_author_tutorial.md)
2. [`docs/write_your_first_custom_analyzer.md`](docs/write_your_first_custom_analyzer.md)
3. [`docs/release_and_compatibility_policy.md`](docs/release_and_compatibility_policy.md)
4. [`docs/external_consumer_checklist.md`](docs/external_consumer_checklist.md)

## Examples

The shipped example set is now:

1. [`examples/scripted_repo_search_workflow.py`](examples/scripted_repo_search_workflow.py)
   - offline analyzer authoring with `WendaoArrowSession.for_repo_search_testing(...)`
2. [`examples/attachment_pdf_analyzer_workflow.py`](examples/attachment_pdf_analyzer_workflow.py)
   - scripted-by-default PDF attachment search over Rust-returned rows, with optional endpoint mode
3. [`examples/document_extraction_workflow.py`](examples/document_extraction_workflow.py)
   - Docling-backed multi-format document extraction into Arrow resource rows, fixture mode by default
4. [`examples/repo_search_workflow.py`](examples/repo_search_workflow.py)
   - host-backed repo-search analysis with built-in `score_rank`
5. [`examples/custom_repo_analyzer_workflow.py`](examples/custom_repo_analyzer_workflow.py)
   - host-backed repo-search analysis with a custom analyzer object
6. [`examples/host_backed_repo_search_beta_smoke.py`](examples/host_backed_repo_search_beta_smoke.py)
   - one-shot beta smoke for the full host-backed repo-search path

Example commands:

```bash
uv run python examples/scripted_repo_search_workflow.py
uv run python examples/attachment_pdf_analyzer_workflow.py
uv run python examples/document_extraction_workflow.py
uv run python examples/repo_search_workflow.py --help
uv run python examples/custom_repo_analyzer_workflow.py --help
uv run python examples/host_backed_repo_search_beta_smoke.py --mode custom --port 0
```

Document extraction performance can be measured through the Rust ignored test
harness, driven by the local benchmark script:

```bash
uv run python tests/scripts/benchmark_wendao_document_extract.py
```

Reports are written to the script's configured report directory. The default
benchmark uses a fake converter for deterministic fixture coverage. Real
Docling coverage is opt-in. The benchmark can prepare a sparse local fixture
checkout so the real run uses Docling's own `tests/data` attachments without
adding those files to this repository:

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

Use `--skip-audio` when audio model artifacts should not be loaded. For real
audio conversion, install `documents-audio` and run without `--skip-audio`; the
benchmark configures the bundled `imageio-ffmpeg` executable for media
conversion. Pass
`--python-uv-extra documents` for real Docling document OCR and
`--python-uv-extra documents-audio` for real audio ASR worker starts.
For the production audio-shard path, run the same harness with
`--flight-mode audio-shards` so the Rust provider or Gateway plans and
materializes timeline shards, sends `xiuxian_wendao.audio_shard_input.v1`
batches over `/analysis/audio-shards`, and merges
`xiuxian_wendao.audio_shard_result.v1` rows through the Rust precision gate.
Use `--audio-worker hosted` for the production OpenAI-compatible audio path
and `--audio-worker docling` only as an explicit comparator. The managed
Wendao analyzer startup selects hosted OpenRouter audio by default; local
Qwen3-compatible testing uses the same hosted worker with an OpenAI-compatible
local base URL. Use `--audio-workers` to cap analyzer-side request
concurrency, and the `--rust-audio-*` flags to profile model-neutral Rust
chunking, materialization, base/recovery worker budgets, and optional
speech-timestamp recovery controls.
When a VAD or speech-density sidecar exists, pass
`--rust-audio-speech-segments-jsonl <segments.jsonl>` with optional
`--rust-audio-speech-merge-gap-ms`, `--rust-audio-speech-min-window-ms`, and
`--rust-audio-speech-limit-chunks`; the harness forwards them to the Rust
provider or Gateway, which constrains failed-row recovery planning before the
unchanged `/analysis/audio-shards` Flight call.
Use `--only-fixture audio` or another fixture name for targeted real fixture
diagnostics. Use `--docling-source-root` only when you already have a prepared
Docling fixture checkout. Use `--concurrency` to stress the Rust-to-Python
cache-hit path and `--server-start-timeout` for cold Docling starts. The report
captures request counts, wall-clock timing, Arrow IPC bytes, status counts, and
error-row counts; `--fail-on-error-rows` makes table-shaped conversion failures
fail the benchmark run.
Use `--fail-on-structure-order-mismatch` when a real OCR benchmark must fail
if force, shard-cache rebuild, and cache-hit artifacts produce different
structure order signatures.
Use `--fail-on-structure-parity-mismatch` when a real OCR benchmark has a
Docling baseline and must fail if the candidate loses baseline text coverage,
protected block counts, or any other structure parity guard.
Use `--fail-on-precision-gate-failure` when a benchmark candidate should fail
on the aggregate precision gate instead of only recording the failure in the
report. This combines error rows, artifact errors, structure order, structure
parity, and Docling groundtruth status from `precisionSpeedSummary`.
Each report also includes a `precisionSpeedSummary` section that keeps the
quality and latency signals together: error rows, artifact errors, structure
order, force/cache/shard-reuse order stability, parity status, OCR/bbox block
counts, force latency, cache p95, shard-cache rebuild latency, Rust scheduler
elapsed time, Docling convert time, Docling convert share, and request-boundary
overhead share. These fields make it explicit whether a cold miss is dominated
by Docling parsing or by Rust/Flight/Arrow overhead.
Full-document Docling cache misses also write an internal
`_document_metrics.arrow` sidecar. It records phase timings for conversion,
Markdown export, resource row construction, structure sidecar construction,
and Arrow cache writes. The sidecar is benchmark evidence only; it does not
change the returned resource table, `_resources.arrow`, `_structure.arrow`,
OCR `_metrics.arrow`, or the Flight/REST extraction contracts.
The document Flight service pre-initializes the Arrow table and IPC writers at
startup so the first user request does not pay the one-time PyArrow writer
initialization cost. Reports also compute document timing overhead as
`forceRefreshMs - documentTimingTotalElapsedMs` when a timing sidecar exists,
which separates Python extraction work from Flight/Rust request-boundary cost.
`precisionSpeedSummary.maxDoclingConvertShare` and
`precisionSpeedSummary.maxDocumentTimingOverheadShare` keep those two costs
visible in the same precision gate used for PDF, image, Office, XML, audio, and
structured-text regressions.
Mixed Docling fixture runs also include `summary.attachmentClassSummary` plus
an `Attachment Class Summary` Markdown table. The class summary groups the same
precision and speed signals by attachment class, including PDF, Office,
images, structured text, web documents, table data, XML, subtitles, audio,
Docling JSON, archive-backed documents, and unknown custom inputs. This keeps
non-PDF attachment regressions visible without changing the Arrow Flight
document extraction contract. Class summaries also aggregate resource type
counts, structure block type counts, bbox block counts, Rust image audit
format counts, dimension-source counts, known-dimension counts, acceleration
candidate counts, and the slowest force/cache fixture in each class so image
OCR and XML/table-heavy hotspots can be diagnosed before adding a new fast
path. Rust image audit is a preflight/control-plane signal only; Docling
remains the image OCR and layout authority until a later parity-gated fast path
is proven.
For `hybrid-page-ocr`, pass `--shard-cache-reuse-probe` when you need explicit
evidence that OCR shard cache reuse works independently from the
whole-document `_resources.arrow` cache. The probe runs a second forced
extraction into a fresh output directory after the first force run and reports
`shardCacheReuseForceMs` in JSON and Markdown output. Reports also include an
`ocrShardCache` summary with the shard cache root, Arrow file count, total
bytes, and configured limits. Hybrid OCR reports also summarize the internal
`_metrics.arrow` sidecar: shard metric row count, OCR result characters, bbox
coverage count, and Rust scheduler elapsed time. Hosted VLM/OCR benchmark
reports include `hostedVlmPromotionGate`, which checks precision, row/order
stability, character floor, hosted request success, hosted key presence,
force-refresh latency, and shard-cache reuse against the locked
`fast-risk-window` promotion baseline. This is a reporting gate only; it does
not change runtime routing.
The Rust provider defaults the OCR shard cache limit to 10 GiB and supports
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES`,
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES`, and
`WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS` for deployment-specific
capacity policy. Local benchmark runs isolate the OCR shard cache by default
so fake and real OCR evidence do not cross-contaminate; use
`--ocr-shard-cache-root` when a run intentionally targets a persistent cache.
For non-PDF and full-document cache validation, pass
`--artifact-registry-reuse-probe`. That probe runs `force=false` into a fresh
output directory after the force run and reports `artifactRegistryReuseForceMs`,
which measures Rust content-hash artifact mirroring separately from the normal
same-output cache-hit path. In local mode this probe starts the Rust Flight
provider even when `--flight-mode sync` is selected, because Python-direct sync
benchmarks do not exercise the Rust artifact registry.
Use `--structure-baseline-root` when a golden Docling baseline artifact root is
available. The Rust ignored benchmark reads each candidate `_structure.arrow`,
matches it to the baseline artifact directory by fixture output name, and
reports strict structure parity pass/error fields without changing the
document extraction Flight contract. Pass `--generate-structure-baselines` to
run a sync/full-Docling baseline pass before candidate probes; when no explicit
baseline root is provided, the script writes those baseline artifacts below the
configured report directory.
For source-PDF page-range OCR, Rust owns the outer scheduling policy. It may
split one contiguous source-PDF OCR range into several contiguous subranges and
send those subranges concurrently to Python/Docling. The default source-range
target uses the current adaptive Rust OCR budget, capped by a machine-derived
source-range ceiling and page count because real Docling conversion can regress
when too many page-range conversions run at once. Milestone and regression
gates should leave `--rust-pdf-ocr-source-range-workers` unset so the Rust
scheduler's system-aware auto policy is exercised. Use `--rust-pdf-ocr-workers`
only for the global Rust OCR budget ceiling and
`--rust-pdf-ocr-source-range-workers` only for diagnostic profile sweeps.
For `docling-structure-recovery`, the benchmark auto profile caps Docling
full-profile PDF accelerator threads to one per Python document worker through
`WENDAO_DOCUMENT_EXTRACT_FULL_THREADS=1`. Rust keeps outer parallelism and
endpoint-pool scheduling authority, while each Python worker avoids nested
Docling thread contention. Use `--document-extract-full-threads` only for
diagnostic sweeps or platform-specific validation. The May 8, 2026 DocLayNet
fixture run with this auto policy preserved zero errors, stable order, and
Docling structure parity while reducing cold force refresh to `10127.429667 ms`
against the locked `12856.546292 ms` baseline.
For explicit benchmark readiness, use `--document-extract-prewarm-source-path`
with `--document-extract-prewarm-page-ranges` to run selected Docling page-range
conversions before the worker advertises readiness. This warms the converter
and Docling lazy state only; it does not publish candidate output artifacts or
replace the force-refresh precision gate. The default is disabled. On May 8,
2026, prewarming page range `1:1` for the same DocLayNet PDF fixture preserved
zero errors, stable order, Docling structure parity, `13` resource rows, and
`12` structure blocks while reducing force refresh to a best sample of
`8715.070334 ms`. A repeat of the same shape preserved the same correctness
gates but measured `11627.203583 ms`, so treat prewarm as an explicit readiness
control with visible variance rather than a stable sub-10s default.
For OCR shard workers, `--pdf-ocr-prewarm-source-path` with
`--pdf-ocr-prewarm-page-indices` now stores an in-process source-page Markdown
cache keyed by converter profile, resolved source path, page index, file size,
and file mtime. Matching source-PDF OCR rows can reuse that prewarmed Markdown
through the existing OCR shard result schema and Rust precision gates. The
cache does not publish benchmark output artifacts and does not apply to
rendered-image OCR rows, but it intentionally shifts selected source-page
Docling conversion out of force-refresh timing. Keep it explicit and report the
prewarm fields with every promoted readiness profile.
The current auto policy targets seven source PDF pages per worker before
clamping to the adaptive Rust budget, machine cap, remaining permits, and shard
count. Within that bounded chunk budget, Rust reads a lightweight source-PDF
page complexity profile and forms contiguous, reading-order-preserving
subranges. The planner must not cross cache-miss gaps, and it keeps every
selected page on the source-PDF OCR lane so the Python worker continues to use
Docling over original PDF page ranges rather than lower-precision raster or
table-fast shortcuts.
Python also recognizes the existing `docling-fast-text-ocr` OCR profile as a
separate Docling converter profile. Rows with different `ocrProfile` values are
not merged into one source-PDF page range, so Rust can mix fast and accurate
Docling ranges without changing the Arrow schema. The benchmark exposes this as
`--rust-pdf-ocr-profile-planner fast-risk-window`, which forwards
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER=fast-risk-window` to the Rust
provider. The local real-Docling benchmark server passes `ocrProfile` through
its converter factory, so `docling-fast-text-ocr` uses Docling's FAST table
mode even when benchmark fixture options also configure other formats. Because
Rust owns the outer OCR worker budget, the fast-text Docling converter uses one
internal accelerator thread by default to avoid multiplying Python-side Docling
threads across Rust-scheduled source-range chunks. Set
`WENDAO_PDF_OCR_FAST_TEXT_THREADS` only for a host-specific diagnostic sweep.
The benchmark can also pass
`--pdf-ocr-fast-text-source-converter backend-table`, which forwards
`WENDAO_PDF_OCR_FAST_TEXT_SOURCE_CONVERTER=backend-table` to local Python OCR
workers. This diagnostic source-PDF-only converter keeps Docling FAST table
structure while disabling Docling OCR; rendered-image OCR rows stay on the
normal fast-text converter. The default is `default`, and this mode is not a
promotion candidate without fresh canary evidence because it changes Docling's
real-worker critical path even when isolated page microprofiles look faster.
Python also recognizes `docling-backend-text-ocr-v1`, a PDF-native backend-text
source-range profile that disables Docling OCR and table-structure work while
forcing backend text extraction. Rust can combine that profile with
`docling-fast-text-ocr` top-up pages in the benchmark-only
`hosted-vlm-risk-window-backend-text` planner without changing the Arrow shard
schema. In that mixed source-range mode, Rust schedules contiguous profile
runs as the dispatch unit, prioritizes fast-text top-up runs before backend
runs, and automatically expands the requested dispatch budget to the run count
before clamping it through live worker permits. When the hosted recovery
region pipeline is also in `render-dispatch` mode, Rust keeps the run-count
floor for the remaining source top-up ranges after local backend-text rows are
handled. This lets non-contiguous precision top-up ranges run in parallel with
hosted region recovery while the live Rust scheduler still owns worker
admission. The mode remains opt-in and is promoted only for benchmark evidence
that preserves the frozen character floor and beats the locked force-refresh
baseline.
The benchmark can additionally pass
`--rust-pdf-local-backend-text rust-lopdf` to let the Rust provider satisfy
`docling-backend-text-ocr-v1` rows with the attachment-owned `lopdf`
source-text helper. The current promoted OpenRouter canary does not require
that helper: it uses `mistralai/ministral-3b-2512`, render-dispatch with
render-ahead `3`, region trim, and an explicit `2s` hosted hedge. Two May 9,
2026 milestone runs preserved zero error rows, stable `27` rows, `21/6`
page/region OCR blocks, and the frozen character floor: best force refresh
`7338.796584 ms` with `metricsResultChars=115735`, and repeat force refresh
`8322.027792 ms` with `metricsResultChars=115925`. The hosted request wall
span was `5166 ms` and `6140 ms`; both runs used `12` HTTP attempts for `6`
logical hosted region requests, so this remains an explicit benchmark profile
decision rather than a global default. The older endpoint-local `4s` hedge
sample at `8201.568417 ms` and the 2026-05-07 r59/r60 evidence remain
historical regression controls. A same-shape `1s` hedge canary stayed
precision-valid at `8562.0245 ms` but did not beat the `2s` envelope.
The latest follow-up canaries keep the same active envelope. Disabling
fast-text top-up is rejected because it preserved row/order checks but dropped
`metricsResultChars` to `100981`, below the frozen `103984` floor. Page `5`
prewarm plus `single-page-first` affinity measured `8516.511291 ms` with the
precision gate intact, but did not beat the `8322.027792 ms` repeat. Composite
size `3` reduced hosted requests to `4`, but hosted p95 reached `8528.296 ms`
and force refresh regressed to `10797.20775 ms`. Region render chunk mode
`region` improved first region readiness to about `0.70-0.72s`, but the two
precision-valid repeats measured `8202.969708 ms` and `8927.807167 ms`, so it
remains diagnostic instead of replacing page-grouped region chunks.
Current-rev direct region rasterization is now a separate benchmark control:
pass `--rust-pdf-region-render-mode direct-crop` to forward
`WENDAO_DOCUMENT_EXTRACT_PDF_REGION_RENDER_MODE=direct-crop` to the Rust
provider. The mode keeps render DPI intact and changes only the PDFium region
rasterization path. The May 20, 2026 r21 OpenRouter canary first promoted the
readiness lever at `10430.886708 ms`. The follow-up r22 canary kept direct-crop
and singleton Rust endpoint fanout, then allowed large low-complexity
risk-neighbor regions to keep the pixel-driven adaptive split. It preserved
zero error rows, stable `28` rows, `21/7` page/region OCR blocks,
`metricsResultChars=107616`, and `precisionGatePassed=true`, while reducing
force refresh to `9741.029333 ms`; the report captures
`rustPdfRegionRenderMode=direct-crop`. The r26 repeat kept the stable
page-chunk render shape, ran full shard-cache and artifact-registry probes, and
became the current revision's promoted readiness-overhead candidate:
`8463.964667 ms`, zero error rows, stable `28` rows, `21/7` page/region OCR
blocks, `metricsResultChars=108893`, `maxShardCacheReuseForceMs=138.07075`,
and `maxArtifactRegistryReuseForceMs=139.23179199999998`. The adjacent r20
canary with a `2s` hosted hedge stayed precision-clean but regressed force
refresh to `10966.508541 ms`, so the current direct-crop candidate keeps the
`5s` hedge. Benchmark runs that pass `--require-pdfium` now treat Rust PDFium
render fallback as a hard error before full Docling fallback rows can be counted
as hosted-region performance evidence.
`region-seed-page` is the next opt-in chunk-shape canary: Rust renders the
smallest recovery region first, then renders the remaining regions grouped by
page. The analyzer sees the same OCR shard rows and request schema; promotion
still depends on the normal row/order, character-floor, hosted-tail, and
precision gates. May 21, 2026 milestone canaries rejected the current
`region`, `region-seed-page`, dispatch-size `3`, and endpoint-reservation
variants as promotion paths: all preserved zero errors, stable `28` rows,
`21/7` page/region OCR blocks, and the frozen character floor, but force
refresh stayed between `11899.885791 ms` and `12775.375916 ms`. The measured
regression came from source-range fast-text chunks stretching past `9s-12s`,
so the next optimization surface is lane-level scheduler fairness, not more
region chunk splitting.
`--rust-pdf-ocr-scheduler-lane-fairness source-first` is the first scheduler
fairness canary. It forwards
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SCHEDULER_LANE_FAIRNESS=source-first` so Rust
lets source-PDF page-range OCR groups enter the Python Flight endpoints before
rendered-region fanout when both lanes are present. Defaults stay unchanged.
The first r32 attempts were invalid startup or PDFium-gate runs, not
OCR-positive performance evidence. The r32d rerun is the valid source-first
sample: zero error rows, stable `28` rows, `21/7` page/region OCR blocks,
`metricsResultChars=115704`, and `precisionGatePassed=true`. Its
`10874.933208 ms` force refresh beats the locked `12856.546292 ms` baseline,
but it is rejected as the current promoted canary because r26 remains faster at
`8463.964667 ms`. The r32d trace moved the critical path to source-range
fast-text chunks, with pages `6:10` reaching `10237.833084 ms`; further gains
need a separate source-range tail slice rather than more hosted-region chunk
splitting. The benchmark harness now forwards `--require-pdfium` to live Rust
providers and gateways as `WENDAO_PDF_RENDER_REQUIRE_PDFIUM=1`, so future
fail-fast canaries cannot silently count full-Docling fallback rows as
hosted-region performance.
Follow-up r33b restored `--rust-pdf-local-backend-text rust-lopdf` and proved
backend-text source rows are not the remaining bottleneck: those rows dropped
to about `239 ms`, while Docling fast-text page `5` tailed at
`10929.931542 ms`. r36 fixed the opt-in `single-page` fast-text split budget
so pages `5`, `11`, `12`, and `13` are dispatched in the same wave, but force
refresh still measured `12330.44625 ms` because page `5` itself stretched to
`11892.002583 ms`. The current promotion therefore stays on r26; the next real
optimization has to preserve Docling fast-text coverage while reducing the
page-level conversion cost.
The analyzer now stores OCR prewarm source-page Markdown in an in-process
fingerprinted cache keyed by converter profile, resolved source path, page
index, file size, and file mtime. Source-PDF OCR requests reuse those prewarmed
rows only when every fingerprint matches; otherwise they fall back to the
normal Docling conversion path. This keeps Docling fast-text as the authority
while shifting known slow page readiness out of force-refresh timing. The May
21, 2026 r40 milestone canary kept zero error rows, stable `28` rows, `21/7`
page/region OCR blocks, `metricsResultChars=108850`, and
`precisionGatePassed=true`; source-range max chunk time fell to `280.219583 ms`
and force refresh measured `9037.577666 ms`. r26 remains the fastest current
revision baseline at `8463.964667 ms`, so prewarm-cache reuse is an opt-in
readiness lever rather than a global default.
`--rust-pdf-local-backend-text-empty fail-fast` is a diagnostic scheduler
canary for source-page-range placeholder rows. When Rust `lopdf` proves a
`docling-backend-text-ocr-v1` source page has empty backend text, or cannot
produce the requested source-page text vector, the provider returns a failed
OCR row immediately so the precision-preserving
full-document fallback can run without sending the non-image placeholder
through the Python raster OCR path. The default remains `dispatch-python`.
`--pdf-ocr-backend-text-empty-page verified-empty` is a narrower recovery
canary for true empty source-page-range rows. The benchmark forwards it to the
Python worker as `WENDAO_PDF_OCR_BACKEND_TEXT_EMPTY_PAGE=verified-empty` and
to the Rust provider as
`WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE=verified-empty`. When a
`docling-backend-text-ocr-v1` source-page placeholder has no image path and
backend-text plus the optional compatible-page retry both produce no text, the
worker may return a successful empty Markdown row. Rust accepts that empty text
only for backend-text page shards backed by source-page-range placeholders, so
ordinary empty OCR output still triggers the existing precision fallback.
The 2026-05-08 r108c real Docling canary rejected this mode as a promotion
candidate for the `pdf-redp5110-sampled`, `pdf-skipped-1page`, and
`pdf-skipped-2pages` fixtures: force refresh improved, but all three structure
parity checks failed because backend-text rows did not preserve baseline text
coverage. Use the structure parity gate for any future run that enables this
canary.
`--pdf-ocr-backend-text-page-fallback compatible-page` is a separate Python
worker canary for backend-text page failures. It retries only failed or empty
backend-text source pages through `docling-compatible-page-ocr-v1` before the
Rust provider escalates to full-document fallback. The default remains
`disabled`; promotion requires structure parity and Docling groundtruth gates
because compatible page Markdown may not preserve full-document structure.
`--rust-pdf-backend-text-topup disabled` is a separate character-floor
diagnostic. It is rejected for the current milestone fixture because disabling
top-up drops `metricsResultChars` below the frozen floor; the current-rev
canary measured `100981` result characters at `9169.448167 ms`. The default
remains `profile`.
`--rust-pdf-backend-text-topup hosted-vlm` is a full-page hosted VLM/OCR
top-up replacement canary. It is rejected for the milestone fixture because
force refresh regressed to `35374.309 ms` and `metricsResultChars=91265` fell
below the frozen floor; the hosted model did not preserve the dense page `5`
text coverage that Docling fast-text top-up currently supplies.
`--rust-pdf-local-fast-text rust-lopdf` is also diagnostic-only. It improves
source-range latency on the milestone fixture but is rejected because it drops
`metricsResultChars` below the frozen floor. The May 21, 2026 r34 rerun
measured `84886` result characters against the `103984` floor after replacing
Docling fast-text with local `lopdf`; correctness wins, so the mode remains
diagnostic-only. The benchmark precision gate now treats a checked milestone
character-floor regression as a precision failure, not only as promotion-gate
metadata. The converter-only prewarm
diagnostic `--pdf-ocr-prewarm-profile docling-fast-text-ocr` is
precision-valid but did not improve the promoted r59/r60 envelope. Pair it
with `--pdf-ocr-prewarm-source-path` and `--pdf-ocr-prewarm-page-index` when a
long-lived local worker should perform one real source-page conversion before
listening, which triggers Docling's lazy table-structure warmup outside the
force-refresh request path. On the milestone fixture, source page `0` warmup
reduced page `5` fast-text top-up to about `7.2-7.4s` and passed repeat
promotion at `11990.357708 ms` and `11537.015125 ms`, but it is accepted as a
stability diagnostic rather than the current best evidence because it is slower
than the promoted `8201.568417 ms` OpenRouter sample.
Use `--pdf-ocr-prewarm-endpoint-count N` when only the first `N` local OCR
endpoints should receive that source-page prewarm. This avoids the invalid
full-pool page `5` prewarm failure while still allowing Rust endpoint affinity
experiments. The paired
`--rust-pdf-fast-text-endpoint-affinity single-page-first` canary routes
single-page fast-text source-PDF chunks to the first endpoint. On the milestone
fixture, r70 preserved precision and completed force refresh at
`9636.47725 ms`, with page `5` fast-text reduced to `5274.754916 ms`. It is
accepted as endpoint-locality evidence. A 2026-05-08 control without
endpoint-local prewarm and affinity regressed to `22329.780375 ms`, with page
`5` fast-text source-range work tailing at `20193.906625 ms`; restoring the r70
shape brought the canary back to `10164.795292 ms` with a `5s` hosted hedge,
and tightening only the hosted hedge to `4s` produced the previous promoted
`8201.568417 ms` sample. Current-rev `2s` hedge repeats now supersede it as
the active OpenRouter region-recovery envelope. r71 prewarmed endpoints `0-3`
and reduced the page
`11-13` fast-text chunk to `5972.05625 ms`, but force refresh regressed to
`10336.721667 ms` because the hosted region tail dominated.
The later Ministral same-page region composite size `3` diagnostics preserved
precision but remain rejected. The older page `12` three-region composite
request tailed at `14430.981 ms` and force refresh regressed to
`17806.492208 ms`; the current-rev repeat reduced hosted requests to `4` but
still measured `10797.20775 ms` force refresh with hosted p95 `8528.296 ms`.
The r83 composite repeat with source split and endpoint affinity failed before
OCR metrics with a Flight `BrokenPipe`, so region composite remains a
provider-stability canary rather than a promoted OpenRouter optimization. The
worker now traces unexpected composite exceptions as failed canary attempts
and falls back to individual region requests, preserving the existing row
contract. The r84 fallback-guard rerun completed with valid OCR metrics and
passed the locked baseline at `12658.151 ms`, with `metricsResultChars=106410`,
zero error rows, stable `27` rows, and `21/6` page/region OCR blocks. It
remains diagnostic only because it is slower than the promoted `8201.568417 ms`
OpenRouter envelope.
`--rust-pdf-fast-text-source-range-split single-page` is a source-range
chunk-shape diagnostic. It keeps Rust scheduler permits as the admission
owner and preserves precision on the milestone fixture. The older r64b run is
rejected because page `5` single-page Docling fast-text conversion regressed
force refresh to `23629.474667 ms`. The later r82b run, paired with
endpoint-local fast-text affinity, passed the locked promotion gate at
`12133.964875 ms` with `metricsResultChars=108788`, but it remains
diagnostic-only because it did not beat the current promoted OpenRouter sample
and page `5` still dominated the source-range tail. The May 21, 2026 r35/r36
pair closed a scheduler artifact in this diagnostic mode: r35 showed pages
`12` and `13` waiting for page `5`; r36 expanded the opt-in split budget so all
four fast-text pages entered Flight in the first wave. The corrected r36 shape
preserved rows and character floor (`metricsResultChars=108376`) but still
measured `12330.44625 ms`, so extra fast-text concurrency is not promoted. The
r37 repeat combined that budget repair with endpoint-local prewarm and
fast-text affinity. It preserved precision at `10148.07225 ms`,
`metricsResultChars=109481`, zero error rows, stable `28` rows, and `21/7`
page/region OCR blocks, but it still did not beat the r26 promoted baseline.
The backend-table converter diagnostic is also rejected for this milestone:
r38b preserved precision but regressed force refresh to `22977.377709 ms`, and
r39 preserved precision but regressed to `24330.926125 ms`. Both runs kept the
page `5` Docling fast-text source chunk as the dominant tail, so backend-table
stays a no-go diagnostic rather than a default or promoted source-range path.
The r40 default-converter check kept backend-table disabled and reused the
explicit source-page prewarm cache for the known fast-text pages. It passed the
same precision gates with `metricsResultChars=108834`, zero errors, stable
`28` rows, and `21/7` page/region OCR blocks, while reducing force refresh to
`8051.183125 ms`. Treat r40 as the current explicit-prewarm OpenRouter
readiness envelope; cold profiles without source-page prewarm must be compared
separately. A same-shape r41 canary with a shorter `3s` hosted speculative
retry delay is rejected: it stayed precision-clean but regressed force refresh
to `8839.854667 ms` and expanded hosted HTTP attempts to `27`. Its force
timing shows first hosted region readiness at `2131.201625 ms`, while
shard-cache reuse shows the same region path ready in `34.896-51.849167 ms`;
the remaining local lever is cold region render readiness, not shorter hosted
hedging. A follow-up r42 all-region render chunk canary is also rejected:
precision stayed clean, but force refresh regressed to `14896.950292 ms`,
first hosted region readiness moved to `7726.35725 ms`, and source-range
backend-text chunks stretched to the `4786-7465 ms` range.
The later render-shape canaries kept the same precision contract but did not
improve the r40 envelope. r46b `region-seed-page` is rejected because force
refresh regressed to `16500.03625 ms` and first hosted region readiness moved
to `5206.039334 ms`. r47 re-ran the promoted page-grouped direct-crop shape
with expanded render telemetry; it stayed below the locked `12856.546292 ms`
gate at `10821.847375 ms`, with `metricsResultChars=108556`, zero errors,
stable `28` rows, `21/7` page/region blocks, three render chunks, three region
dispatches, and `9141.296209 ms` reported cold region-render work. r48
`default` page-raster-plus-crop is rejected because it did not improve force
refresh (`10829.041292 ms`) and increased reported cold render work to
`10700.698043 ms`. Keep page-grouped `direct-crop` as the current reference;
the next local optimization should reduce cold PDFium region-render readiness
without serializing hosted request startup.
The fast
profile is still not the global default because broader corpus promotion needs
separate evidence, but the arXiv `2604.17337` guard proves this opt-in
risk-window profile on the current machine: 12,856.546 ms force-refresh
latency, 92.084 ms shard-cache reuse latency, 2.309 ms cache p95, zero error
rows, 21 OCR page blocks, 21 bbox blocks, 21 metrics rows, 103,985 OCR result
characters, and stable structure order.
Python recognizes `hosted-vlm-direct-ocr-v1` as the model-agnostic direct VLM
OCR profile over the same shard rows. That path calls an externally managed
OpenAI-compatible backend, with vLLM as the preferred runtime, using
`WENDAO_HOSTED_VLM_OCR_BASE_URL`, `WENDAO_HOSTED_VLM_OCR_MODEL`,
`WENDAO_HOSTED_VLM_OCR_API_KEY`, `WENDAO_HOSTED_VLM_OCR_PROMPT`,
`WENDAO_HOSTED_VLM_OCR_MAX_TOKENS`,
`WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS`,
`WENDAO_HOSTED_VLM_OCR_REGION_PROMPT_MODE`, and
`WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS`. Set
`WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY` to tune direct remote request
fan-out; keep it provider-gated by benchmark evidence because some hosted
models lose stability when concurrency is too high. The direct client retries
transient hosted HTTP failures such as `429`, `500`, `502`, `503`, and `504`
with a short bounded backoff before returning a failed row, so Rust precision
and fallback gates still own the final decision. Set
`WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE` above `1` to let the direct worker
combine contiguous page images into one request as a provider capability
canary. Current hosted optimization keeps this disabled by default. The
`hosted-vlm-risk-window` planner sends only the source-profile risk window to
the hosted VLM/OCR backend while leaving ordinary pages on the fast profile.
`hosted-vlm-risk-window-backend-text` is a benchmark-only source-range
optimization canary that keeps hosted recovery semantics but routes ordinary
low-risk pages to `docling-backend-text-ocr-v1` and dense text top-up pages to
`docling-fast-text-ocr`.
The Docling-centered recovery lane makes this package the structure execution
owner, not just a full-document fallback. The internal Flight metadata header
`x-wendao-document-extract-page-range` requests a 1-based inclusive Docling
conversion range and writes uniquely prefixed page-range resource rows plus a
matching `_structure.arrow` sidecar. Studio may merge those rows back by
`pageIndex` to replace failed or empty page OCR rows while preserving Docling
reading order. Hosted OpenRouter, local OCR2, backend text, and fast text are
therefore text patch or acceleration paths over Docling structure, not
replacement structure pipelines.
Rust may request multiple contiguous page ranges for one document when a
benchmark enables page-range chunking. The analyzer treats each range as an
independent Docling conversion request with stable page-range element ids;
Studio is responsible for wrapper-row normalization, structure parity checks,
and escalation to full-document fallback when any range is incomplete. The
benchmark report now keeps per-range Docling fallback timing and the slowest
chunk summary visible so analyzer conversion cost can be separated from Rust
scheduler and Flight overhead.
For tail-latency diagnostics, `--rust-pdf-docling-page-range-chunk-plan`
forwards an exact 1-based fallback chunk plan such as `1:3,4:4,5:6,7:9` to the
Rust provider. The Rust side rejects plans that omit, duplicate, or include
pages outside the Docling fallback set, so this remains a precision-preserving
benchmark control rather than a new routing default. The first May 8, 2026
tail-splitting canary with that plan preserved Docling structure parity but
regressed to `18912.534209 ms`, so the accepted default remains three-page
chunks.
For automatic high-cost diagnostics,
`--rust-pdf-docling-page-range-structure-cost-budget` forwards
`WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET` to Rust.
When set, Rust may split automatic `docling-structure-recovery` fallback ranges
whose source-profile structure cost exceeds the budget. The split may spend
spare document-extract endpoint capacity, but it must remain a single Docling
execution wave; without spare capacity Rust keeps the capped range shape. The
flag is disabled by default and is only a benchmark-visible diagnostic control;
structure parity, row order, and precision gates still decide promotion.
Set `WENDAO_DOCUMENT_EXTRACT_CONVERTER_CACHE=profile` only for explicit
page-range benchmark probes that need to test whether reusing a Docling
converter across Flight document-extract calls reduces conversion setup cost.
The benchmark flag is `--document-extract-converter-cache profile`. The default
remains disabled so ordinary document extraction keeps the existing converter
lifecycle.
Set `WENDAO_DOCUMENT_EXTRACT_PREWARM_SOURCE_PATH` and
`WENDAO_DOCUMENT_EXTRACT_PREWARM_PAGE_RANGES` only through benchmark-controlled
readiness probes. These controls are intentionally separate from converter
reuse: prewarm validates daemon readiness and Docling lazy initialization,
while converter-cache probes isolate repeated page-range setup cost after the
worker is already serving requests.
Explicit `region-shards` benchmarks are the narrower
recovery-surface proof before automatic region discovery is promoted. Narrow
exact-risk-only page routing is not the promotion path because the real
milestone run lost the frozen character floor. In the current mixed-render
route, Rust keeps ordinary pages as source-range rows and renders 300 DPI page
images only for hosted VLM recovery pages, preserving the existing Arrow input
and result schemas. When the benchmark supplies explicit region shards, Rust
keeps the parent page on the fast source-range profile and appends recovery
region rows as supplemental recovery inputs. Rust binds each hosted recovery
region row to the retained fast parent page and records a `sentinel-sidecar-v1`
structure provenance marker so region recovery remains a validated sidecar
patch rather than an implicit string splice. When no explicit region JSON is configured,
`--rust-pdf-hosted-vlm-region-planner profile-risk-window` lets Rust derive a
conservative content-band region for pages already selected by
`hosted-vlm-risk-window`; this remains benchmark-only until region precision and
stitching pass the promotion gate. The adjacent benchmark-only
`profile-risk-window-slices` planner splits that same content band into
top/middle/bottom same-page regions so hosted VLM/OCR tests can exercise
same-page composite requests without changing the Arrow shard schema.
`profile-risk-window-adaptive` keeps the same benchmark-only source selection
but chooses one, two, or three slices from Rust's source-page structure profile
and estimated content-band pixel area. Exact structure-risk pages may receive
more slices, while low-complexity risk-window neighbor pages can stay as one
region. It is the preferred next hosted benchmark profile because it targets
the measured trade-off between single-band provider tail latency and blanket
three-slice request overhead without lowering DPI.
The benchmark can pass `--rust-pdf-hosted-vlm-region-target-pixels` and
`--rust-pdf-hosted-vlm-region-max-slices` to tune Rust's adaptive region patch
sizing for hosted VLM/OCR canaries. The flags forward to Studio environment
controls, preserve the default planner when omitted, and are reported in JSON,
promotion evidence, and Markdown output. Smaller slices are treated as
diagnostic until they reduce hosted wall span while preserving row order,
precision gates, and the unchanged Arrow shard schema.
The benchmark can also pass
`--rust-pdf-hosted-vlm-region-pipeline render-dispatch`, forwarded to
`WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE`, to let Rust overlap
ordinary source-range OCR scheduling with hosted recovery-region rendering.
This mode is a local-overhead optimization probe for the measured gap between
hosted request wall span and force-refresh latency; it stays disabled by
default and does not change the Arrow shard schema or precision gates.
Pass `--rust-pdf-hosted-vlm-region-render-ahead N` when that opt-in pipeline
should pre-render more than one page-region chunk while hosted requests are in
flight. Rust still sorts the final OCR shard inputs back to deterministic
reading order before validating rows and writing result artifacts.
When this flag is omitted, Rust owns the default worker-window decision:
render-dispatch reserves one endpoint for the base OCR batch, uses remaining
endpoint capacity for region render-ahead, and clamps by planned render chunks.
The benchmark report records planned render chunks, endpoint count, effective
render-ahead, and render spawn count alongside the existing render and dispatch
timings.
Pass `--rust-pdf-region-render-mode direct-crop` when testing PDFium direct
region rasterization for hosted recovery rows. This is independent from render
chunk ordering, keeps DPI unchanged, and is reported in both JSON and Markdown
benchmark output so promotion evidence can identify the rasterization path.
Pass `--rust-pdf-hosted-vlm-region-render-chunk region` only as a chunk-shape
diagnostic. It asks Rust to render each recovery region as an independent
chunk rather than grouping regions by page, so the first hosted request can be
dispatched as soon as the first region is ready. Current-rev milestone repeats
preserved zero errors, stable order, `27` rows, `21/6` page/region blocks, and
the frozen character floor while moving first region readiness to about
`0.70-0.72s`; force refresh still measured `8202.969708 ms` and
`8927.807167 ms`, so the default remains page-grouped chunks. The all-region
render chunk diagnostic is also rejected because it delayed first hosted
dispatch to `12528.588583 ms` and regressed force refresh to `21670.3075 ms`.
The current direct-crop milestone repeats confirmed the same boundary:
`region` reached first ready at `1066.33125 ms` but regressed force refresh to
`12775.375916 ms`; `region-seed-page` reached first ready at `766.841416 ms`
but still regressed to `11899.885791 ms`; dispatch-size `3` and an
endpoint-reservation canary also stayed above `12s`. These are rejected because
they protect accuracy but starve source-range fast-text work.
Pass `--rust-pdf-hosted-vlm-region-render-chunk region-seed-page` for the
middle-ground canary: one smallest-area region becomes an early seed request,
and all remaining recovery regions stay page-grouped. This is intended to test
whether early hosted dispatch can be recovered without the full tail cost of
splitting every region. The first explicit PDFium OpenRouter gate passed the
precision gate with zero error rows, stable `27` rows, `21/6` page/region OCR
blocks, and the frozen character floor. The measured pair is
`8250.492790999999 ms` with `metricsResultChars=116286`, then
`8445.105417 ms` with `metricsResultChars=116270`. It remains a canary because
it has not beaten the active `7338.796584/8322.027792 ms` envelope and cannot
replace page-grouped chunks yet.
Region rows use
`WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS`,
defaulting to 2048 and clamped by `WENDAO_HOSTED_VLM_OCR_MAX_TOKENS`, so a
single hosted region response cannot silently consume the full page-token
budget unless the benchmark explicitly raises the region cap through
`--hosted-vlm-ocr-region-max-tokens`. Set
`WENDAO_HOSTED_VLM_OCR_REGION_PROMPT_MODE=compact-region-markdown`, or pass
`--hosted-vlm-ocr-region-prompt-mode compact-region-markdown`, to make
single-region requests tell the hosted model that the input is a cropped
recovery patch, not a full document page. The mode is disabled by default and
does not change the OCR shard schema, DPI, result schema, row order, or
precision gates; it is a prompt/output-shaping canary for reducing hosted
remote-tail variance while preserving visible text, tables, formulas, symbols,
and reading order. Set
`WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE` above `1`, or pass
`--hosted-vlm-ocr-region-composite-size`, to let the direct worker combine
same-page, same-parent hosted recovery region rows into one multi-image request. Region
composite output must split back into one non-empty Markdown result per region
sentinel marker; otherwise the worker falls back to individual region requests
so the existing row/order contract is preserved. Batched page-window responses
follow the same marker-split rule for page markers. The benchmark can also set
`--rust-pdf-hosted-vlm-region-dispatch-chunk-size N` when a composite canary
must co-dispatch multiple rendered region rows to one Python worker. Leave that
Rust dispatch knob unset for endpoint fanout: Python composite sizing alone no
longer changes Rust dispatch grouping, so adaptive composite rejection cannot
accidentally serialize same-page region requests behind one worker.
The benchmark can also set
`WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE=same-page-json`, or pass
`--hosted-vlm-ocr-region-atlas-mode same-page-json`, to pack a same-page region
composite group into one labeled PNG atlas and request strict JSON keyed by
exact shard markers. Atlas mode is an opt-in request-surface canary: valid JSON
is canonicalized back into Markdown rows, while invalid JSON, row-count
mismatches, empty text, and HTTP failures fall back to individual region
requests so the existing Arrow shard result contract and precision fallback
remain unchanged. Set
`WENDAO_HOSTED_VLM_OCR_SCAFFOLD_MODE=region-table-json`, or pass
`--hosted-vlm-ocr-scaffold-mode region-table-json` in the benchmark harness, to
enable structural scaffold recovery for hosted recovery region rows. In that mode the
worker loads Studio's `_hosted_vlm_region_scaffolds.json` sidecar beside the rendered
region images, validates the shard id, parent shard id, source content hash,
and raster hash, asks the hosted VLM/OCR provider for JSON-only output keyed by exact region markers,
and canonicalizes valid table/text JSON back into Markdown result rows. Missing
sidecars, fingerprint mismatches, malformed JSON, marker or row-count
mismatches, empty canonical text, and invalid table cell shapes return failed
rows so the existing Rust precision fallback protects correctness.
The current OpenRouter/Qianfan fast benchmark rejects scaffold composite as a
promotion path because the provider failed strict row-count and canonical-text
validation in real runs. Keep scaffold mode disabled unless the selected
provider has fresh evidence with zero scaffold validation failures. Set
`WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS` above `0`, or pass
`--hosted-vlm-ocr-speculative-retry-delay-seconds`, to enable an opt-in
hedged request for direct hosted region rows: if the first hosted request has
not produced a valid result by that delay, the worker starts a second request
and returns the first valid Markdown response. This is a tail-latency guard for
cloud OCR providers; it does not change the Arrow schema or validation gates,
and benchmark traces report both logical requests and HTTP attempt counts so
request-cost evidence stays visible. The older May 9 OpenRouter envelope pins
this delay at `2s`, while the current revision's direct-crop r26 canary pins
it at `5s`; r41 shows that tightening the current explicit-prewarm shape to
`3s` duplicates hosted attempts without improving the force-refresh tail, so it is
diagnostic-only. These are benchmark evidence, not global default changes. Set
`WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_SOURCE_PIXELS` or
`WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_IMAGE_BYTES`, or pass the matching
benchmark flags, to skip speculative retries for small direct region shards
while keeping the fixed delay for larger tail-risk regions. These thresholds
default to disabled, do not change OCR rows, and must be promoted only with
precision-clean wall-span evidence.
Set
`WENDAO_HOSTED_VLM_OCR_TRACE_PATH` to a JSONL file when a benchmark needs
request-level latency, HTTP status, image-byte, Markdown character,
shard-type, region-count, render-DPI, source-pixel-area, scaffold mode,
scaffold applied count, scaffold validation failure count, JSON character
count, and canonical Markdown character telemetry. Trace records intentionally
omit API keys and image payloads. Hosted VLM/OCR recovery
benchmarks must not lower render DPI to gain speed; region shrinkage and
provider capability gates are the accepted optimization levers. Set
`WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION=region-whitespace-trim`, or benchmark
with `--hosted-vlm-ocr-image-optimization region-whitespace-trim`, to trim
near-white margins from Hosted VLM/OCR region PNG request payloads. The default
is `disabled`; the optimization applies only to region rows, keeps render DPI
intact, and does not change Arrow shard rows, result schema, row order, or
validation gates. Benchmark trace summaries report
`imageOptimizationModeCounts`, and the top-level Hosted VLM/OCR report records
the selected mode. Analyzer declares `Pillow` as a runtime dependency because
region trimming and same-page atlas packing both need deterministic image
decoding outside Docling's optional dependency set.
Scheduler trace summaries also expose source-range queue wait and dispatch
start/end timing when the Rust scheduler provides those fields. These are
diagnostic-only report fields used to separate request latency from scheduler
admission and wave ordering; they do not change the Arrow OCR shard input or
result schemas.
A May 8, 2026 real-fixture diagnostic separated fixture coverage from hosted
OCR evidence. The Docling `2305.03393` page and full-paper probes validated
the real fixture path but routed only to backend-text source-page shards under
the risk-window planner. A forced DocLayNet region through
`baidu/qianfan-ocr-fast:free` preserved zero errors and stable order, but the
single hosted region request tailed at about `16.7s` while scheduler queue wait
was effectively zero. For that provider, the next latency lever is model or
provider choice, region payload shape, or scaffolded output constraints rather
than Rust queue admission.
The analyzer normalizes wrapping quotes from Hosted VLM/OCR environment values
before building OpenRouter headers. This protects local `.env` loaders that
materialize a value such as `"sk-or-v1-..."` with literal quotes. With that
normalization in place, the same real DocLayNet region completed through
`mistralai/ministral-3b-2512` with zero errors, stable order, one hosted
region request at about `7.4s`, and a force-refresh time around `9.3s`.
`qwen/qwen3-vl-8b-instruct` also preserved correctness on the same probe but
tailed around `27.3s`, so it remains a rejected candidate for this region.
Set
`WENDAO_HOSTED_VLM_OCR_PROVIDER=openrouter` to use OpenRouter's
OpenAI-compatible `/chat/completions` API instead of a local model server. The
OpenRouter preset defaults the base URL to `https://openrouter.ai/api/v1`,
reads `WENDAO_OPENROUTER_API_KEY` or `OPENROUTER_API_KEY`, accepts
`WENDAO_OPENROUTER_MODEL` when `WENDAO_HOSTED_VLM_OCR_MODEL` is not set, and
forwards optional
`WENDAO_OPENROUTER_HTTP_REFERER` and
`WENDAO_OPENROUTER_TITLE` attribution headers. The selected OpenRouter model
must support image URL chat content. When no OpenRouter model is configured,
the hosted smoke default is `baidu/qianfan-ocr-fast`, which is used only to
validate the cloud OCR path; current promotion evidence should pin the faster
validated `mistralai/ministral-3b-2512` candidate explicitly. A May 20, 2026
private LTC probe found the free OpenRouter aliases for Qianfan OCR had no
available live endpoints, while the non-free `baidu/qianfan-ocr-fast` route
successfully produced queue-keyed image OCR JSONL for the same adapter path.
For hosted-region tail studies, set
`WENDAO_HOSTED_VLM_OCR_OPENROUTER_PROVIDER_JSON` to a JSON object that should
be sent as the OpenRouter request-body `provider` field. This is opt-in route
control for experiments such as latency sorting or provider ordering; it does
not change the OCR shard Arrow schema, rendered DPI, prompt contract, or
default OpenAI-compatible payload. The benchmark forwards the same value with
`--hosted-vlm-ocr-openrouter-provider-json` and records the provider routing
object in JSON and Markdown reports.
Use the
[OpenRouter quickstart](https://openrouter.ai/docs/quickstart) when configuring
the hosted provider:

```bash
export WENDAO_HOSTED_VLM_OCR_PROVIDER=openrouter
export WENDAO_OPENROUTER_API_KEY=...
export WENDAO_OPENROUTER_MODEL=baidu/qianfan-ocr-fast
export WENDAO_HOSTED_VLM_OCR_OPENROUTER_PROVIDER_JSON='{"sort":{"by":"latency"}}'
```

The analyzer does not load or quantize OCR2 weights in process. Use the
repository `fetch-models` and `start-ocr-backend` Justfile recipes only when a
local backend is desired; they fetch prebuilt community or official artifacts
and expose them through a vLLM OpenAI-compatible service:

```bash
just fetch-models
just start-ocr-backend
```

`fetch-models` also auto-selects the local artifact flavor. On macOS Apple
Silicon it defaults to the MLX-converted
`mlx-community/DeepSeek-OCR-2-bf16` artifact and links it to
`deepseek-ocr2-current`; on non-Mac hosts it defaults to the community FP8
vLLM artifact. Set `WENDAO_DEEPSEEK_OCR2_MODEL_FLAVOR=generic-vllm` or
`metal-mlx`, set `WENDAO_DEEPSEEK_OCR2_HF_REPO`, or pass `repo_id=...` when
validating a newer AWQ, GPTQ, GGUF, or MLX artifact. The Justfile recipes are
thin entrypoints over the analyzer CLI (`wendao-document-extract --ocr2-*`),
so backend automation stays package-owned and directly testable.
`start-ocr-backend` auto-selects the local platform runner. On macOS Apple
Silicon it selects `mlx-vlm`, which wraps the MLX-converted OCR2 model in a
repository OpenAI-compatible adapter. Install the shared local Metal/MLX
runtime and probe it with:

```bash
just install-vllm-metal
just probe-vllm-metal
```

The Metal runner exports the MLX/Metal defaults (`VLLM_METAL_USE_MLX=1`,
`VLLM_MLX_DEVICE=gpu`, and
`VLLM_METAL_MULTIMODAL_MODE=multimodal-native`) before invoking the vLLM CLI.
The `mlx-vlm` runner is the default on macOS because a direct local probe with
`mlx-community/DeepSeek-OCR-2-bf16` successfully returns OCR text through the
OpenAI-compatible `/v1/chat/completions` shape expected by the analyzer. The
lower-level `metal-vllm` runner remains available for explicit vLLM Metal
experiments, but it still gates DeepSeek-OCR-2 by default because current vLLM
Metal documentation lists multimodal vision/audio models as unsupported and
focuses the plugin on text-only language models. Set
`WENDAO_DEEPSEEK_OCR2_VLLM_METAL_ALLOW_UNSUPPORTED_VLM=1` only for explicit
frontier probes after validating the local vLLM Metal build. The current local
frontier probe reaches Metal engine initialization and resolves
`DeepseekOCR2ForCausalLM`, but the vLLM Metal path loads VLMs in text-only mode
and fails inside the `mlx-vlm` DeepSeek vision tower before serving an
endpoint. The immediate compatibility mismatch is that the OCR2 artifact uses a
DeepEncoderV2-style nested `vision_config.width`, while the current
`mlx-vlm` DeepSeek-VL2 loader expects a scalar vision width.
`vllm-omni` is not selected as the default OCR2 backend in this slice: its
current supported-models surface does not list DeepSeek-OCR-2, and it does not
address the macOS Metal `mlx-vlm` vision-loader mismatch above.
On non-Mac deployment hosts, `start-ocr-backend` selects `generic-vllm` and
starts the current vLLM OpenAI-compatible server shape for OCR2. It adds the
OCR2 n-gram logits processor and disables prefix caching by default, matching
the current vLLM OCR2 serving guidance. `WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE`
defaults to `vllm>=0.20.1`, so repository automation requires a vLLM line with
first-class OCR2 serving support instead of silently resolving to older
packages. Deployment hosts that need a known CUDA wheel or exact compatibility
set can still pin `WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE`, add helper packages
through `WENDAO_DEEPSEEK_OCR2_VLLM_WITH`, or override all server flags with
`WENDAO_DEEPSEEK_OCR2_VLLM_EXTRA_ARGS`.
`WENDAO_DEEPSEEK_OCR2_BACKEND_RUNNER=official-vllm` remains available as a
diagnostic compatibility runner for the official repository adapter, but it is
not the default because that adapter follows older vLLM internal APIs.
Default analyzer installs stay free of vLLM and model-runtime dependencies. The
`docling-vlm-deepseek-ocr` profile remains a Docling VLM comparator path, not
the default direct OCR2 implementation.
Production deployments can set
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS` directly when local
evidence shows a fixed override is appropriate for that machine profile, but
that override is not the default correctness or performance gate.
Pass `--fixture-suite milestone --only-fixture autosearch-2604.17337` for the
repo-owned OCR milestone input. Use `--fixture-suite explicit` with
`--extra-fixture NAME=PATH` only when benchmarking another audited real input
without preparing the full Docling fixture checkout. Milestone inputs that
define regression gates must come from a repo-tracked fixture path or another
explicit, auditable source path; `.data` downloads are cache material only. Add
`--fail-on-pdf-milestone-regression` to fail an OCR-positive milestone run when
it drops below the stored `2604.17337` precision/speed envelope. The guard is
evaluated after the JSON and Markdown reports are written so a failing run
still leaves evidence for diagnosis. Low `metricsResultChars` is recorded as a
milestone regression, not as a missing milestone observation.
The May 5, 2026 source-range endpoint-fanout profile kept the default Rust
source-range worker policy unset, used the default local endpoint auto fanout,
and observed a 21-page force latency of 18,969.021 ms with zero error rows,
21 OCR page blocks, 21 bbox blocks, and 103,984 OCR result characters. A
same-machine diagnostic four-endpoint run observed 15,811.373 ms, so endpoint
pool fanout is the current larger optimization lever; fixed source-range worker
counts remain diagnostic only. A later May 5, 2026 structure-order weighted
multi-shard run kept the same defaults and observed 16,364.335 ms force
latency, 92.843 ms shard-cache rebuild latency, 2.261 ms cache p95 latency,
zero error rows, 21 OCR page blocks, 21 bbox blocks, 21 metrics rows, 103,984
OCR result characters, and stable structure order. That run is still above the
sub-15 target, but it preserves the precision envelope while improving the
accepted default fanout baseline. The follow-up profile-aware fast-risk-window
run stayed on automatic endpoint fanout, kept the Rust source-range worker
override unset, and passed the stored milestone guard with 12,856.546 ms force
latency, 92.084 ms shard-cache reuse latency, 2.309 ms cache p95 latency, zero
error rows, 21 OCR page blocks, 21 bbox blocks, 21 metrics rows, 103,985 OCR
result characters, and stable structure order.
Use `--local-python-ocr-endpoint-count N` when a local benchmark should start
`N` Python Flight executors, including the primary document worker, and expose
that pool to the Rust scheduler for both full-document conversion and PDF OCR
shards. The default `auto` value keeps ordinary modes at one local endpoint and
fans out real `hybrid-page-ocr` Docling OCR by the machine profile so the Rust
source-range endpoint-pool scheduler is exercised without pinning a fixed worker
count. Use `--rust-document-extract-endpoint` or `--rust-pdf-ocr-endpoint` more
than once when a benchmark should target already-running Python Flight
executors. The script forwards those endpoints through
`WENDAO_DOCUMENT_EXTRACT_ENDPOINTS` and
`WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS`; when these flags are omitted,
Rust keeps using the normal document extraction endpoint as the single Python
executor.
For async provider validation, run with `--flight-mode async`; the driver starts
the synchronous Python worker plus the existing Rust Flight provider and can
verify cold duplicate-miss deduplication with `--duplicate-miss-concurrency`
and `--fail-on-duplicate-conversions`. Use `--distinct-miss-concurrency` to
add a suite-level burst where different documents cold-miss at the same time;
that report section records queue depth, running conversions, permit pressure,
and one-conversion-per-document counts when a converter counter is available.
For real Docling fixtures, run distinct-miss and duplicate-miss probes as
separate benchmark invocations so both remain true cold-miss measurements.
When a Rust gateway REST endpoint is already running, pass
`--rust-rest-endpoint http://127.0.0.1:<gateway-port>` to sample
`GET /api/document-extract-jobs` during each cargo probe. The JSON and Markdown
reports then include queue depth, running job count, in-process scheduled jobs,
conversion capacity, permit pressure, and last/max conversion duration.
For a fully local gateway-observed run, use `--rust-provider-mode gateway`.
The benchmark starts a temporary Valkey process, the synchronous Python worker,
and `wendao gateway start`, then samples the gateway REST status endpoint while
the Rust Flight probes run:

```bash
uv run python tests/scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --skip-audio \
  --only-fixture pdf \
  --only-fixture image-png \
  --only-fixture uspto-xml \
  --only-fixture xbrl-xml \
  --flight-mode async \
  --rust-provider-mode gateway \
  --wait-ms 70000 \
  --duplicate-miss-concurrency 4 \
  --fail-on-duplicate-conversions \
  --fail-on-error-rows
```

For the distinct-document cold-miss capacity slice, run a separate invocation
with `--distinct-miss-concurrency` and no duplicate-miss flag:

```bash
uv run python tests/scripts/benchmark_wendao_document_extract.py \
  --real-docling \
  --fixture-suite docling-real \
  --skip-audio \
  --flight-mode async \
  --rust-provider-mode gateway \
  --wait-ms 70000 \
  --distinct-miss-concurrency 4 \
  --fail-on-distinct-miss-conversions \
  --fail-on-error-rows
```

The cache-hit path is optimized for service use: cached `_resources.arrow`
files are returned as Arrow tables without a Python row roundtrip, and the Rust
document extraction provider reuses its Tonic channel for the configured
endpoint. Sync `force=false` extraction also checks the Rust content-hash
artifact registry before calling Python; when a complete artifact already
exists, Rust mirrors it into the requested output directory and returns the
Arrow table without re-running Docling. Successful sync conversions are mirrored
back into that artifact registry for future reuse. First-time Docling
conversion is still CPU/model bound and should be handled with queueing and
worker-pool sizing in production. The Rust provider limits concurrently
running cold conversions with host `available_parallelism()` by default.
Full-profile Docling conversion runs in a child Python process by default, so
native crashes in heavyweight model paths fail the current extraction instead
of terminating the Arrow Flight worker. Set
`WENDAO_DOCUMENT_EXTRACT_FULL_ISOLATION=false` only for local diagnostics that
need inline stack traces. Set `WENDAO_DOCUMENT_EXTRACT_FULL_TIMEOUT_SECONDS`
when a deployment needs a stricter wall-clock bound than the default 900
seconds. Attachment-oriented `fast-text` extraction remains inline and uses the
lighter profile selected by the Rust or frontend request header.
`WENDAO_DOCUMENT_EXTRACT_MAX_RUNNING_CONVERSIONS` is a deployment upper bound:
set it when a host should run fewer simultaneous Docling conversions than its
CPU budget would otherwise allow. Jobs waiting for this Rust-owned capacity
remain in `queued` status and do not occupy Python-side SQL or registry work.
The browser-facing `GET /api/document-extract-jobs` endpoint returns the Rust
provider runtime snapshot, including queue counts, running counts, conversion
capacity, scheduled in-process jobs, and last/max conversion duration. Use it
with the single-job `GET /api/document-extract-job?job_id=...` status endpoint
when tuning real PDF/OCR/audio pressure runs.
The Python service stays a synchronous Arrow Flight worker. Async queueing,
content-hash deduplication, job status, and DuckDB-backed durability are owned
by the Rust Wendao provider/gateway so Python does not spend cycles on SQL
registry work before returning Arrow data.
Deployments can run multiple Python document workers and set
`WENDAO_DOCUMENT_EXTRACT_ENDPOINTS` to a comma- or whitespace-separated pool.
Rust round-robins full-document cache misses across that pool while keeping
content-hash deduplication, queue state, and artifact mirroring in the Rust
control plane. `WENDAO_DOCUMENT_EXTRACT_ENDPOINT` remains the single-endpoint
fallback and the default endpoint used when the pool is not configured.

For
[RFC: Polyglot Compute Orchestrator](../../../docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md)
Phase 1.1, this package owns only the Python Docling Flight worker surface:
document conversion, OCR shard execution, Arrow resource rows, and the existing
`/analysis/document-extract` and `/analysis/pdf-ocr-shards` service routes.
Rust continues to own queueing, worker-budget selection, status metadata,
cache policy, and fallback coordination. The approved
`xiuxian-polyglot-orchestrator` crate may model Python-lane admission and
pressure evidence, but it does not add a second Docling wrapper service or
change the analyzer public route/schema contract.
The Rust-side harness profile verifies the owner bridges and Studio adoption
point; this Python package remains covered by its package-local Python project
harness and pytest suites.

## Beta Readiness

ready now:

1. offline scripted repo-search authoring
2. scripted PDF attachment analysis over Rust-returned attachment-search tables
3. Docling-backed local multi-format document extraction into Arrow resource rows
4. Wendao-facing Arrow Flight service entrypoint for document extraction
5. host-backed repo-search analysis with built-in or custom analyzers
6. generic rows/table/query analysis over Rust-returned data
7. package-local Python project harness coverage for syntax, project shape,
   modularity, pytest layout, and agent-policy drift

known gaps before broader adoption:

1. no GA-level release promise yet
2. no analyzer-owned rerank helper surface
3. callers still need `uv run python ...` from the package directory for the
   shipped examples

## Beta Freeze Audit

The current package boundary is now intentionally lockable as `0.2.1`.

frozen for this beta trial:

1. the document extraction service route and returned Arrow resource schema
2. the six shipped examples above
3. `WendaoArrowSession.for_repo_search_testing(...)` as the documented offline
   author workflow
4. `WendaoArrowSession.attachment_search(...)` plus `run_table_analysis(...)`
   as the documented PDF attachment workflow seam
5. `extract_document_table(...)` as the documented multi-format document parsing workflow seam
6. `wendao-document-extract` as the Wendao-facing service command
7. `run_repo_analysis(...)` and `run_query_analysis(...)` as the host-backed
   analyzer entrypoints
8. the rule that analyzer-owned rerank helpers are out of scope

not frozen for this beta trial:

1. helper symmetry for every possible Wendao route
2. future analyzer strategies beyond `score_rank`
3. additional convenience wrappers over already materialized transport results

current freeze rule:

1. workflow-frozen, not helper-frozen
2. transport-owned rerank remains in `wendao-core-lib` and
   `wendao-arrow-interface`

## Beta Exit Audit

exit-ready now:

1. real-host repo-search coverage exists through `wendao_search_flight_server`
2. offline authoring is available through the scripted session surface
3. docs and examples align with the Rust-query-first analyzer boundary

not exit-ready yet:

1. no GA-level versioning promise yet
2. no broader downstream feedback cycle yet
3. no committed compatibility window beyond this beta baseline
