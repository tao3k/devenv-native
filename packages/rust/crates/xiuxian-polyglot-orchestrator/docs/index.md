# xiuxian-polyglot-orchestrator: Map of Content

:PROPERTIES:
:ID: 13052f07971da8ccfaba82df50bcbf863e68cd91
:TYPE: INDEX
:STATUS: ACTIVE
:END:

Standardized documentation index for the `xiuxian-polyglot-orchestrator`
package.

This package owns thin Rust control-plane contracts for the Wendao polyglot
compute lane. Runtime behavior, worker lifecycle, route ownership, schema
ownership, and transport construction remain in the existing Wendao owner
packages. The crate may compute inert Docling scheduling plans from supplied
facts, but owner packages still translate those plans into existing route,
header, batch, cache, ordering, and fallback behavior. Studio now consumes the
plan for the common OCR worker/shard clamp while retaining live dispatch and
pressure observation authority.

Module surfaces:

1. `lanes`: lane identity and capability classification.
2. `admission`: admission budget and decision contracts.
3. `evidence`: health, readiness, pressure, and fallback evidence.
4. `pressure`: worker budget, queue, failure, and ordering pressure evidence.
5. `docling_schedule`: pure document-extraction and OCR-shard scheduling
   plans derived from owner-supplied pressure evidence.
6. `readiness`: Julia profile, route, schema, manifest, warmup, and benchmark
   readiness evidence.
7. `schema_benchmark`: advisory schema-strategy benchmark evidence and report
   contracts.
8. `refs`: typed route/profile/schema owner references.
9. `snapshot`: inert read-only aggregation of owner-provided control-plane
   facts.

Harness profile:

1. `src/lib.rs` owns the crate-level public API profile.
2. `src/docling_schedule/model.rs` owns the Docling scheduling-contract
   profile.
3. `tests/unit/lib` owns source-backed unit coverage mounted by the crate
   root.

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-05-05
:END:
