# xiuxian-wendao-attachments

`xiuxian-wendao-attachments` owns reusable attachment parsing, audit, and
artifact helpers for Wendao document surfaces.

The crate is intentionally not part of Wendao's default feature set. Expensive
or native document tooling stays behind explicit features so the main Wendao
gateway can depend on the crate without pulling PDF accelerators into default,
`studio`, or `performance` builds.

## Features

| Feature         | Purpose                                                                      |
| --------------- | ---------------------------------------------------------------------------- |
| `pdf-inspector` | Enables the pinned `tao3k/pdf-inspector` audit and text-layer proof helpers. |
| `pdf-render`    | Enables PDFium-backed page rendering and OCR shard manifest helpers.         |

## Boundaries

- `xiuxian-wendao-attachments` owns optional PDF accelerator dependencies such
  as `pdf-inspector` and `pdfium-render`.
- `xiuxian-wendao` owns the Studio gateway, Flight/REST routes, and production
  document extraction behavior.
- Production extraction still falls back to Python/Docling unless a later
  approved milestone wires a feature-gated fast or hybrid path into the live
  provider.
- The stable document extraction resource table remains Arrow-based. Browser
  JSON is only an edge serialization surface.

## Test Policy

This crate depends on `xiuxian-testing` and mounts the shared crate test-policy
harness from `src/lib.rs`. Unit tests live under `tests/unit/` and are mounted
back into the source modules with `#[path]` so focused `cargo test --lib`
commands still run the relevant tests.
