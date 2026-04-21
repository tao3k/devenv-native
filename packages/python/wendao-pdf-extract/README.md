# wendao-pdf-extract

Arrow Flight PDF extraction service for Wendao, powered by [OpenDataLoader](https://github.com/ai4os/opendataloader).

## Overview

This service exposes PDF extraction via an Arrow Flight endpoint (`/analysis/pdf-extract`).
It converts PDFs into structured resources:

- **Markdown** — full document text (`{stem}.md`)
- **Images** — per-page extracted images (`page_{N}_img_{K}.{ext}`)
- **Tables** — HTML table markup embedded in JSON metadata
- **Formulas** — LaTeX math embedded in JSON metadata

Extracted assets are written to `{pdf_path}.extracted/` so the Wendao VFS naturally
picks them up.  The Arrow response returns only lightweight metadata rows.

## Prerequisites

- Python 3.10+
- Java 11+ (required by OpenDataLoader's underlying PDF parser)

## Installation

```bash
pip install opendataloader-pdf
pip install -e .
```

## Usage

### Start the server

```bash
wendao-pdf-extract --host 0.0.0.0 --port 50051
```

### Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WENDAO_PDF_EXTRACT_ENDPOINT` | `http://localhost:50051` | Rust gateway target endpoint |

## Arrow Flight Contract

### Request (metadata headers)

- `x-wendao-schema-version`: `v2`
- `x-wendao-pdf-extract-source-path`: absolute or repo-relative path to the PDF
- `x-wendao-pdf-extract-output-dir`: output directory (default: `{source_path}.extracted`)
- `x-wendao-pdf-extract-images`: `true` / `false`
- `x-wendao-pdf-extract-tables`: `true` / `false`
- `x-wendao-pdf-extract-formulas`: `true` / `false`

### Response schema

| Column | Type | Description |
|--------|------|-------------|
| `sourcePath` | utf8 | Source PDF path |
| `resourceType` | utf8 | `document` / `image` / `table` / `formula` / `error` |
| `resourcePath` | utf8 | Path to extracted file (empty for inline content) |
| `pageIndex` | int32 | 1-based page number |
| `caption` | utf8 | Optional caption |
| `content` | utf8 | Inline content (HTML, LaTeX, text) |
| `mimeType` | utf8 | MIME type of the resource |
| `status` | utf8 | `ok` / `skipped` / `error` |
| `elementId` | utf8 | Stable element identifier |

## Cache Behaviour

The service skips re-extraction when `{output_dir}/_complete.marker` exists and is
newer than the source PDF.  Delete the `.extracted/` directory to force re-extraction.

## License

Apache-2.0
