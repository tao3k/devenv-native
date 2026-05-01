---
type: knowledge
metadata:
  title: "wendao-core-lib"
---

# wendao-core-lib

`wendao-core-lib` is the transport-first Python package for the
`xiuxian-wendao` Rust runtime.

It exists to give Python consumers a stable Wendao substrate entrypoint while
the repository keeps Python transport ownership narrow and explicit.

The default architecture is:

1. Rust owns execution and state.
2. Python plugin authors implement analyzers against Arrow tables.
3. Python prefers Arrow Flight transport.
4. Python falls back to Arrow IPC.
5. Python does not depend on in-process Rust bindings.

Package positioning:

1. `wendao-core-lib` is a typed Python access layer for Rust-owned Wendao
   transport contracts.
2. `wendao-core-lib` should be treated as the Python-side analogue of
   `.data/WendaoArrow`: it owns Arrow/Flight transport helpers and typed
   contract access, not analyzer logic.
3. a future sibling package such as `xiuxian-wendao-analyzer` should depend on
   `wendao-core-lib` and own Python-local analyzer strategies, scientific
   ecosystem integration, and analyzer runtime configuration.
4. downstream users who want to build custom analyzers over Arrow tables
   should depend on `wendao-core-lib` directly as the shared Python
   substrate.
5. `repo-search` and `rerank` support in this package mean Python can call the
   Rust-owned Flight routes with stable request and response shapes.
6. `repo-search` and `rerank` do not make this package a Python-local search
   or scoring runtime.
7. new semantics should be owned by Rust first and then surfaced here as thin
   typed transport helpers.

Related package-boundary RFC:

- [Python Wendao Analyzer Package Boundary](../../../docs/rfcs/2026-03-31-python-wendao-analyzer-package-rfc.md)

Transitional note:

- the current `analyzer.py`, `plugin.py`, and `scaffold.py` surfaces remain
  available for authoring compatibility, but new Python-local analyzer
  strategy work should move toward the planned `xiuxian-wendao-analyzer`
  sibling package instead of expanding `wendao-core-lib`.
- transport and typed contract access remain first-class here; analyzer
  strategy growth does not.
- the planned analyzer package is expected to reuse the typed transport entry
  points from `wendao_core_lib.transport` instead of rebuilding raw
  Arrow/Flight request assembly.

## Quick Start

```python
from wendao_core_lib import (
    run_analyzer,
    WendaoAnalyzerPlugin,
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
    WendaoFlightRouteQuery,
)

config = WendaoTransportConfig(
    endpoint=WendaoTransportEndpoint(host="127.0.0.1", port=50051),
)
client = WendaoTransportClient(config)
query = WendaoFlightRouteQuery(route="/search/repos/main")

def analyzer(table, context):
    return {
        "rows": table.num_rows,
        "route": context.query.normalized_route(),
        "flight_info": context.flight_info,
    }

result = run_analyzer(client, analyzer, query)
print(result)
```

For the stable repo-search query contract, Python now also exposes a typed
helper layer instead of only `route -> Table` primitives:

```python
from wendao_core_lib import repo_search_query
from wendao_core_lib import repo_search_request

query = repo_search_query()
request = repo_search_request("rerank rust traits", limit=25)
info = client.get_repo_search_info(request)
rows = client.read_repo_search_rows(request)

print(info.endpoints[0].ticket.ticket)
print(rows[0].doc_id, rows[0].path, rows[0].score)
```

Attachment search follows the same transport-owned pattern:

```python
from wendao_core_lib import attachment_search_request

request = attachment_search_request(
    "architecture",
    limit=5,
    ext_filters=("pdf",),
    kind_filters=("pdf",),
)
info = client.get_attachment_search_info(request)
rows = client.read_attachment_search_rows(request)

print(info.endpoints[0].ticket.ticket)
print(rows[0].attachment_name, rows[0].attachment_ext, rows[0].score)
```

For the stable rerank exchange route, Python now also exposes a typed request
builder instead of requiring downstream authors to hand-build Arrow tables:

```python
from wendao_core_lib import WendaoRerankRequestRow

response = client.exchange_rerank_rows(
    [
        WendaoRerankRequestRow(
            doc_id="doc-0",
            vector_score=0.5,
            embedding=(0.1, 0.2, 0.3),
            query_embedding=(0.4, 0.5, 0.6),
        )
    ]
)
print(response.to_pylist())
```

For the currently landed Rust Flight transport seam, Python also supports
table round-trips through `do_exchange(...)`:

```python
import pyarrow as pa

request = pa.table({"id": ["doc-0"], "score": [0.5]})
response = client.exchange_query_table(
    WendaoFlightRouteQuery(route="/rerank/flight"),
    request,
    extra_metadata={"x-wendao-rerank-embedding-dimension": "3"},
)
print(response)
```

That exchange path is now covered by a real Python↔Rust smoke test against a
minimal Rust Flight server example, so the `do_exchange(...)` seam is verified
across runtime boundaries instead of only through Python-side fakes.

The same Rust example now also backs the query-side contract:
`get_flight_info(...)` and `read_query_table(...)` are verified against a live
Rust-owned `FlightDescriptor + Ticket + do_get(...)` path, not only through
Python-side monkeypatch tests.

That query route is no longer just a test-local string: the Rust runtime now
publishes a transport query-contract surface for the stable schema-version
header plus canonical routes such as `/search/repos/main` and
`/rerank/flight`, and the live example/test path consumes that shared
contract instead of duplicating route literals.

The repo-search query path now also carries a stable request+response
contract. Requests now include typed metadata for:

- `x-wendao-repo-search-query`
- `x-wendao-repo-search-limit`

and responses carry the canonical columns:

- `doc_id`
- `path`
- `title`
- `best_section`
- `match_reason`
- `navigation_path`
- `navigation_category`
- `navigation_line`
- `navigation_line_end`
- `hierarchy`
- `tags`
- `score`
- `language`

and the Python package exposes `WendaoRepoSearchResultRow` plus
`WendaoRepoSearchRequest`, `repo_search_request(...)`,
`get_repo_search_info(request)`, `read_repo_search_rows(request)`, and
`parse_repo_search_rows()` so downstream analyzer authors do not need to
hand-parse raw Arrow tables or hand-assemble repo-search request metadata for
the common repo-search case. The live Rust query route now rejects blank query
text and non-positive limits instead of treating `/search/repos/main` as a
parameterless read.
The same response contract now also carries stable backend-owned evidence
fields, `best_section`, `match_reason`, `navigation_*`, `hierarchy`, and
`tags`, so analyzers can inspect the strongest matching section, the concrete
editor-navigation landing point, and the path-derived hierarchy without
reverse-engineering search semantics from path or score alone.

The same package now also exposes the stable rerank request columns:

- `doc_id`
- `vector_score`
- `embedding`
- `query_embedding`

through `WendaoRerankRequestRow`, `build_rerank_request_table(...)`, and
`exchange_rerank_rows()`, so downstream authors can send typed rerank requests
over the live Rust-owned Flight exchange route without manually assembling
Arrow request tables. The request builder now also fail-fast validates that
all `embedding` and `query_embedding` vectors share one fixed dimension before
the batch is converted into Arrow fixed-size-list columns. On the Rust side,
the same exchange route now also expects a rerank-dimension metadata header,
and the Python typed helper path attaches that header automatically. The live
route now also supports `top_k` as a stable rerank response-limit header, and
the Python typed helper path can request top-ranked truncation without manual
Flight metadata assembly. Malformed raw `top_k` headers are rejected by the
live Rust host with the same invalid-argument path exposed to direct Flight
clients. The live
Rust route now also validates the first non-empty exchange schema frame, so
manual `exchange_query_table(...)` callers must still satisfy the stable
rerank schema if they target `/rerank/flight` directly. That schema validation
now lives in the shared Rust transport contract instead of only inside the
mock Flight example, so the schema gate is owned by the same surface that
publishes the canonical rerank column names and metadata headers.
The same live Rust mock route now also enforces row-level rerank semantics
after schema decode succeeds: blank `doc_id` values are rejected, non-finite
`vector_score` values are rejected, and `query_embedding` must remain stable
across all rows in one rerank request batch. The Python↔Rust smoke now covers
that semantic rejection path directly, so downstream authors see the same
Rust-owned error surface whether they use the typed Python helper or upload a
manually built but otherwise schema-valid rerank batch.
That semantic gate now also rejects duplicate `doc_id` values inside one
rerank candidate batch and constrains `vector_score` to the inclusive range
`[0.0, 1.0]`, so `/rerank/flight` has started to express stable candidate
semantics instead of only transport integrity.
The rerank route now also publishes a stable response contract with columns:

- `doc_id`
- `vector_score`
- `semantic_score`
- `final_score`
- `rank`

and Python exposes `WendaoRerankResultRow`, `parse_rerank_response_rows()`,
and `exchange_rerank_result_rows()` so downstream analyzer authors do not need
to hand-parse rerank outputs from generic Arrow tables.
The same rerank response contract now also exposes the raw vector score and
the normalized semantic score that feed the shared Rust scorer, so downstream
analyzers can inspect why one candidate outranked another instead of only
seeing the blended `final_score`.
That rerank scorer is no longer fixed at one transport-local blend. The Rust
runtime now exposes a shared `RerankScoreWeights` policy with a default
`0.4 / 0.6` vector/semantic split, and both `wendao_flight_server` and
`wendao_search_flight_server` now accept runtime overrides through:

- `WENDAO_RERANK_VECTOR_WEIGHT`
- `WENDAO_RERANK_SEMANTIC_WEIGHT`

The live Python↔Rust smoke now proves that changing those env vars changes the
returned `final_score`, ordering, and `rank` on the real Flight host without
changing the typed Python request surface.
On the `xiuxian-wendao` host path, the same weight policy now also resolves
through Wendao retrieval runtime config, so
`link_graph.retrieval.julia_rerank.vector_weight` and
`link_graph.retrieval.julia_rerank.similarity_weight` can drive the Flight
rerank scorer without relying only on process-local env overrides. The real
Python↔Rust host smoke now covers both paths:

- env-driven weight overrides
- workspace `wendao.toml`-driven weight overrides
- when both are present, `wendao.toml` wins over env
  The same real host path now also resolves the expected Flight schema version
  from `link_graph.retrieval.julia_rerank.schema_version` in workspace
  `wendao.toml`, so the live host can track retrieval-policy schema changes
  without requiring a process-local positional override. The
  `wendao_search_flight_server` binary now accepts an explicit
  `--schema-version=<value>` CLI override when a host needs to pin schema
  expectations directly.

Current runtime host settings contract:

- [Wendao Flight Runtime Host Settings Contract](../../../docs/contracts/wendao-flight-runtime-host-settings-contract.md)

This keeps `wendao.toml` as the normal workspace-owned source of truth for the
SearchPlane-backed host, while explicit CLI schema pinning remains available
for bounded host bring-up and test control across both current Flight hosts.

## Arrow Flight Support Matrix

The current `wendao-core-lib` Arrow Flight surface is treated as follows.

| Surface                         | Current support                                                                                                                            | Real-host validation                                                       | Status               |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | -------------------- |
| `repo-search` request contract  | typed request helpers for query text, `limit`, `language_filters`, `path_prefixes`, `title_filters`, `tag_filters`, and `filename_filters` | validated against `wendao_search_flight_server`                            | stable               |
| `repo-search` response contract | typed rows for `doc_id`, `path`, `title`, `best_section`, `match_reason`, `navigation_*`, `hierarchy`, `tags`, `score`, and `language`     | validated against runtime and real search hosts                            | operationally closed |
| `rerank` request contract       | typed rows for `doc_id`, `vector_score`, `embedding`, `query_embedding`, plus optional `top_k`                                             | validated against `wendao_flight_server` and `wendao_search_flight_server` | stable               |
| `rerank` response contract      | typed rows for `doc_id`, `vector_score`, `semantic_score`, `final_score`, and `rank`                                                       | validated against runtime and real search hosts                            | stable               |
| `rerank top_k` semantics        | omitted by default; positive values act as an upper bound; blank, zero, and malformed values are rejected                                  | validated on both current Flight hosts                                     | stable               |
| runtime host settings           | promoted knobs are `rerank weights`, `schema version`, and `rerank dimension` only                                                         | governed by the shared runtime host settings contract                      | temporarily closed   |

Not currently treated as active gaps:

- additional repo-search response fields without new analyzer-facing value
- additional runtime host knobs without shared runtime-policy semantics
- more `top_k` edge-case expansion beyond the current upper-bound and rejection contract

This means the current Arrow Flight line is no longer blocked on basic support.
The remaining work should favor new analyzer-facing query or rerank semantics
instead of more transport-edge hardening on already-closed surfaces.

Interpretation rule:

- `rerank` support in this package should be read as "typed Python access to a
  Rust-owned `/rerank/flight` contract", not "Python implements rerank
  behavior locally".
- future official Python analyzer logic should live in a sibling package such
  as `xiuxian-wendao-analyzer`, with `wendao-core-lib` kept as the shared
  transport substrate for any downstream analyzer package.
  The runtime-owned host now also accepts explicit rerank-dimension pinning
  through `--rerank-dimension=<n>`, and that knob is now governed by the shared
  runtime host settings contract as:
  `explicit CLI override > positional arg > default`.
  The SearchPlane-backed host now follows the same rerank-dimension precedence:
  `explicit CLI override > positional arg > default`.
  That response contract is now also backed by shared Rust-owned validation, not
  just request-side validation. The transport contract now validates rerank
  response schema and batch semantics for:

- non-empty response batches
- non-blank and unique `doc_id`
- finite `final_score` inside `[0.0, 1.0]`
- positive and unique `rank`

The live Rust mock Flight route now validates that outbound rerank batch
through the same shared contract before encoding, so the Python side no longer
depends on example-local response-shape discipline.
The same live route now also derives its rerank response deterministically
from the inbound candidate batch instead of returning a fixed example payload:
responses are sorted by descending `vector_score`, ties break by `doc_id`
ascending, `final_score` reflects the inbound candidate score, and `rank`
starts at `1`. The live Python↔Rust smoke now proves that typed rerank
helpers observe that deterministic route behavior end to end.
That route behavior has now moved one step closer to real rerank semantics:
the shared Rust transport contract scores each candidate by blending inbound
`vector_score` with cosine similarity between `embedding` and
`query_embedding`, so the live `/rerank/flight` mock no longer behaves like a
plain score sorter. The Python↔Rust smoke now proves that semantic alignment
can outrank a higher raw `vector_score` when the embedding match is stronger.
That scoring path is no longer only exercised through the external mock
example: `xiuxian-wendao-runtime` now ships a runtime-owned
`build_scored_rerank_response_batch(...)` helper inside its Flight transport
module, and the crate-local Arrow Flight roundtrip test uses that same helper
to verify a semantic rerank request/response cycle without depending on the
example binary.
That runtime seam is now one step more explicit: the runtime crate exposes a
dedicated `RerankFlightRouteHandler` server-side entrypoint, and both the
external mock Flight example and the crate-local Flight roundtrip test now run
through that handler instead of each wiring rerank exchange behavior on their
own.
That server-side surface is now wider than one route helper: the runtime crate
also exposes a reusable `WendaoFlightService` that owns the stable repo-search
query path plus the rerank exchange path, and the live Rust Flight example now
mounts that runtime-owned service directly instead of shipping an example-local
Flight service implementation.
The crate-local Rust Flight roundtrip now also mounts that same
`WendaoFlightService`, and the runtime-owned Rust Flight client automatically
attaches the rerank embedding-dimension metadata header for `/rerank/flight`,
so both the local Rust roundtrip and the external Python smoke now hit the same
stable rerank exchange contract.
There is now also a runtime-owned entrypoint for that same service surface:
`xiuxian-wendao-runtime` ships a `wendao_flight_server` binary, and the Python
transport smoke can target that binary directly instead of relying on an
example-local server path. Document extraction is served by the
`xiuxian-wendao-analyzer` package through the `/analysis/document-extract`
Arrow Flight route.
That runtime service is no longer hard-wired to one static repo-search batch:
`WendaoFlightService` now accepts a pluggable repo-search provider seam, so a
future `xiuxian-wendao` search backend can be mounted behind the same stable
Flight contract without replacing the transport host itself.
That backend seam is now wired one step deeper into the actual
`xiuxian-wendao` crate: the repository ships a
`SearchPlaneRepoSearchFlightRouteProvider`, a
`build_search_plane_flight_service(...)` helper, and a
`wendao_search_flight_server` binary that mounts `WendaoFlightService`
against the live `SearchPlaneService` repo-content search path for one repo.
That real `xiuxian-wendao` host path now also supports
`WENDAO_BOOTSTRAP_SAMPLE_REPO=1`, which seeds one minimal repo-content sample
into `SearchPlaneService` for local bring-up and integration smokes. Python
now has a direct smoke against `wendao_search_flight_server`, so the
repo-search query path is verified against the real `xiuxian-wendao` binary
instead of only the runtime-owned `wendao_flight_server` binary.
That binary smoke no longer has to rely on startup-time seeding. The search
plane now persists repo-corpus publication metadata to local runtime sidecars
under the repo-search storage root, and the Python smoke can seed sample repo
content once with `wendao_search_seed_sample` before starting
`wendao_search_flight_server` in a fresh process. That gives the
`xiuxian-wendao` Flight host a true persisted-fixture path instead of only an
in-process bootstrap path. The direct Python integration smoke now also
restarts `wendao_search_flight_server` against the same seeded project root,
so the repo-search query contract is verified across repeated host restarts
instead of only one happy-path launch.
The same real `xiuxian-wendao` host path now also backs direct Python rerank
smokes for `/rerank/flight`, including both typed happy-path scoring and the
duplicate-`doc_id` rejection surface. That means the real crate-owned Flight
server now covers both stable query reads and stable rerank exchange behavior,
not only repo-search reads.
The persisted repo-search fixture corpus is now also rich enough to cover
multiple query cases and explicit `limit` behavior on the real host path: the
direct Python smoke now proves one seeded `xiuxian-wendao` project root can
serve distinct repo-search queries with stable path expectations for
`searchonlytoken`, `flightbridgetoken`, and an explicit one-row limit.
The same persisted host path now also locks one concrete ranking rule from the
live repo-content search implementation: when two exact-match candidates tie on
score, the real host returns the lexicographically smaller path first. The
direct Python smoke now proves that `ranktieexacttoken` with `limit=1` returns
`src/a_rank.rs` ahead of `src/z_rank.rs` on the real `xiuxian-wendao` Flight
server.
The repo-search request contract now also carries optional language filters
through `x-wendao-repo-search-language-filters`. Python can pass those filters
through `repo_search_request(..., language_filters=(...))`, and the real
`xiuxian-wendao` host now proves that the same persisted repo corpus can be
filtered down to `README.md` for `markdown` while excluding that row for
`rust`.
The same typed request contract now also carries optional path-prefix filters
through `x-wendao-repo-search-path-prefixes`. Python can pass those prefixes
through `repo_search_request(..., path_prefixes=(...))`, and the real host now
proves that `src/` excludes `README.md` while `src/flight` narrows the seeded
corpus to `src/flight*.rs` rows on the same typed Flight query surface.
The same request contract now also carries optional title filters through
`x-wendao-repo-search-title-filters`. Python can pass those filters through
`repo_search_request(..., title_filters=(...))`, and the real host now proves
that `readme` narrows `alpha` to `README.md` while `flight_search` narrows the
seeded corpus to the matching `src/flight_search.rs` title/path row.
The same request contract now also carries optional tag filters through
`x-wendao-repo-search-tag-filters`. Python can pass those filters through
`repo_search_request(..., tag_filters=(...))`, and the real host now proves
that `lang:markdown` narrows `alpha` to `README.md` while `lang:rust` excludes
that row from the same persisted repo corpus. The same tag surface now also
carries real search semantics from the backend: repo-content exact matches are
published as `match:exact`, and the real host proves that
`repo_search_request(..., tag_filters=("match:exact",))` narrows
`searchonlytoken` to `src/search.rs`. The real host also now proves that
exact-case matches outrank folded-only matches on the same repo-search route:
`CamelBridgeToken` returns `docs/CamelBridge.md` ahead of the lowercase-only
`src/camelbridge.rs`, and the exposed `score` column reflects that ordering:
the exact-case row scores higher than the folded-only row on the same typed
Flight query. The stable repo-search response contract now also carries one
read-side evidence field, `best_section`, so Python consumers can inspect the
backend-selected line/snippet context without reparsing raw Arrow batches.
The same response contract now also carries stable `tags`, so analyzers can
consume backend-owned classification and match semantics directly from Arrow
rows instead of reconstructing them from request filters or frontend JSON.
The same request contract now also carries optional filename filters through
`x-wendao-repo-search-filename-filters`. Python can pass those filters through
`repo_search_request(..., filename_filters=(...))`, and the real host now
proves a case-insensitive exact filename match: `readme.md` narrows `alpha` to
`README.md` without also matching unrelated file names.

## Analyzer Plugin Scaffold

```python
from wendao_core_lib import (
    WendaoAnalyzerPlugin,
    WendaoFlightRouteQuery,
    WendaoTransportClient,
    WendaoTransportConfig,
    WendaoTransportEndpoint,
)

client = WendaoTransportClient(
    WendaoTransportConfig(
        endpoint=WendaoTransportEndpoint(host="127.0.0.1", port=50051),
    )
)
query = WendaoFlightRouteQuery(route="/search/repos/main")

plugin = WendaoAnalyzerPlugin(
    capability_id="repo_search",
    provider="acme.python.analyzer",
    analyzer=lambda table, context: {
        "rows": table.num_rows,
        "route": context.query.normalized_route(),
    },
)

binding = plugin.binding_for_client(client, query)
result = plugin.run(client, query)
print(binding.to_dict())
print(result)
```

## Project Scaffold

```python
from wendao_core_lib import scaffold_analyzer_plugin, write_scaffold_project

files = scaffold_analyzer_plugin(
    package_name="acme-repo-search",
    plugin_id="acme.repo_search",
    capability_id="repo_search",
    provider="acme.python.analyzer",
    route="/search/repos/main",
    sample_template="repo_search",
)

print(files["plugin.toml"])
print(files["src/acme_repo_search/analyzer.py"])
print(files["src/acme_repo_search/cli.py"])
print(files["sample_rows.json"])

written = write_scaffold_project(
    "./acme-repo-search",
    package_name="acme-repo-search",
    plugin_id="acme.repo_search",
    capability_id="repo_search",
    provider="acme.python.analyzer",
    route="/search/repos/main",
)
print([str(path) for path in written])
```

Generated projects now include a small local dev loop:

- one `project.scripts` entry
- one `cli.py` that reads `WENDAO_HOST`, `WENDAO_PORT`, and `WENDAO_ROUTE`
- one direct `plugin.run(...)` path for local analyzer execution
- one offline replay path via `WENDAO_SAMPLE_JSON`
- one mock-Flight replay path via `WENDAO_MOCK_FLIGHT=1`
- one stable `sample_rows.json` contract for offline analyzer development
- selectable sample templates:
  - `docs_retrieval`
  - `repo_search`
  - `code_symbol`
- profile-shaped starter analyzer logic that changes with the selected template

The package also exposes a writer CLI:

```bash
wendao-core-lib-scaffold ./acme-repo-search \
  --package-name acme-repo-search \
  --plugin-id acme.repo_search \
  --provider acme.python.analyzer \
  --profile repo_search
```

Offline replay during analyzer development:

```bash
WENDAO_SAMPLE_JSON=./sample_rows.json acme-repo-search-run
```

Mock-Flight replay with the bundled sample payload:

```bash
WENDAO_MOCK_FLIGHT=1 acme-repo-search-run
```

The highest-level scaffold path is now `--profile`, which links:

- `capability_id`
- Flight route
- sample replay template
- starter `analyzer.py` body
- replay row validation contract
- starter `plugin.toml` metadata

Current starter profiles:

- `docs_retrieval`
- `repo_search`
- `code_symbol`

The scaffolded CLI validates replay rows against the linked profile/template
before running the analyzer, so local sample drift fails fast instead of
silently reaching analyzer logic with the wrong row shape.

The generated `plugin.toml` now also carries a `[starter]` block with
profile-linked metadata such as display name, summary, tags, and the selected
sample template.

That same starter metadata can also travel in runtime binding payloads via the
Python plugin API, so profile semantics are available during registration as
well as during project generation.

The SDK now also exposes helpers to build that runtime starter payload
directly from a named profile, so downstream authors do not need to duplicate
profile metadata by hand when constructing `WendaoAnalyzerPlugin(...)`.

It now also exposes manifest-driven runtime helpers, so scaffolded
`plugin.toml` can feed back into `WendaoAnalyzerPlugin(...)` construction
instead of remaining a generation-only artifact.

That path now includes entrypoint loading as well, so a scaffolded project can
round-trip from generated manifest to loaded analyzer callable to runtime
plugin without hand-written import glue.

The SDK now also validates manifest structure and transport values during the
manifest-driven runtime path, so malformed `plugin.toml` files fail early with
explicit contract errors.

The generated project path is now smoke-testable in mock-Flight mode as a real
CLI entrypoint, not just as a set of library helpers.

That generated CLI is also manifest-driven now: it loads `plugin.toml`,
derives the default Flight route from `plugin.route`, resolves the analyzer
from `[entrypoint]`, and rebuilds the runtime plugin from the manifest so
downstream authors have one source of truth for route and entrypoint changes.

## Scope

- Arrow-backed analyzer authoring helpers that keep Flight descriptor, metadata,
  and table-fetch plumbing out of downstream analyzer code.
- Plugin-binding scaffold aligned to the Rust capability/endpoint/transport
  contract so downstream Python providers do not need to hand-build runtime
  binding payloads.
- Minimal plugin-project scaffold helpers so downstream authors can start from
  a ready-made `plugin.toml`, `pyproject.toml`, analyzer module skeleton, and
  local CLI runner.
- Transport-first package with explicit Flight connection setup and Arrow IPC fallback models.
- `xiuxian_*` canonical Python entrypoint for Wendao transport consumers and plugin authors.
- Rust remains the single source of truth for search/index logic and runtime
  semantics.
