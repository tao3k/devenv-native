# Real Repository Search Precision

:PROPERTIES:
:ID: feat-real-repo-search-precision
:PARENT: [[../index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, search, precision, knowledge, benchmark
:STATUS: ACTIVE
:END:

## Purpose

The real-repository search precision harness validates Wendao search quality
against live repository checkouts instead of synthetic snippets. Its primary
job is to prove that an agent can ask natural-language knowledge questions and
receive compact, source-grounded evidence without moving LLM orchestration
into `xiuxian-wendao`.

The implementation surface lives under
[`src/search/real_repo_precision/`](../../src/search/real_repo_precision/).
The Python-owned profile comparator lives in
[`wendao-knowledge-retrieval-benchmark`](../../../../../../packages/python/wendao-knowledge-retrieval-benchmark/README.md).

## Boundary

The harness is opt-in. It does not add a runtime route, does not change Arrow
schemas, does not change Julia service contracts, and does not commit external
repository checkouts.

`xiuxian-wendao` owns:

- the source-maintained repository catalog and gold expectations
- LinkGraph and repo-AST query evaluation over materialized repositories
- rank-aware receipts, scenario receipts, semantic evidence receipts, and
  benchmark input receipts

External orchestration systems own:

- subagent execution
- LLM calls
- workflow state
- prompt policy
- user-facing search loops

The optional [`pi-wendao`](https://github.com/tao3k/pi-wendao) row is used only
as external orchestration evidence. Wendao proves it can search the backend
facts an orchestrator needs; it does not absorb the orchestrator.

## Current Evidence

The latest multi-repository proof passed `33/33` queries and `9/9` scenarios
over `2` repositories, `480` Markdown documents, and `268474` indexed words.

Repository rows currently prove different search shapes:

- `xiuxian-artisan-workshop`: `27/27` queries and `7/7` scenarios, including
  docs-family knowledge, semantic SSOT evidence, source probes, PageIndex
  seeds, and graph-first reasoning-tree receipts.
- `pi-wendao`: `6/6` queries and `2/2` scenarios over `8` Markdown documents
  and `109` TypeScript AST files, covering subagent host evidence,
  named-workflow evidence, BPMN runtime ownership, and model resolver source.

The docs-corpus proof remains the strongest knowledge-search gate. It passes
`23/23` docs-family queries, `7/7` knowledge scenarios, and `15/15` query
variants over `472` Markdown documents and `263701` indexed words. It records
`31` graph-first disclosure steps, full scenario recall@10, rank facts, and
late-query counts for broad exploratory prompts.

The black-box benchmark compares retrieval profiles over those receipts
without changing Rust ranking behavior. On the larger docs-corpus receipt,
`graph-first-reasoning-tree` keeps the same correctness and reciprocal-rank
quality as `flat-topk`, but reduces exposed path-character cost from `13777`
to `4101`. `intent-tree-v1` preserves the same correctness and exposes `5743`
path characters while recording explicit intent evidence coverage.

On the small `pi-wendao` row, profile scores are intentionally much closer:
flat top-k already finds the required source evidence, and graph-first saves
only a small amount of exposed context. That result is important because it
shows the backend should select a profile by scenario shape instead of
forcing graph-first search everywhere.

## Reflection From Real Validation

The useful improvement is no longer "make every query graph-first." The real
data says there are at least two classes of search:

- known-item and small-repository queries, where flat top-k is cheap and often
  already exact
- evidence-rich knowledge tasks, where relation paths, PageIndex seeds,
  semantic SSOT facts, negative guards, and authority ordering make
  graph-first or intent-tree profiles more reliable and much more token
  efficient

The current harness is doing its job because it exposed both facts. It proved
that graph-first search materially improves evidence coverage on complex
docs-corpus intents, and it also proved that graph-first is not automatically
worth the extra disclosure machinery on tiny or owner-local source questions.

The main remaining problem is not the query inner loop. The live proof still
spends much more time in repository materialization, cache restore, and index
preparation than in profile scoring. For a user-facing agent loop, the right
shape is resident or prewarmed indexes with fingerprint validation, not
rebuilding the proof harness for every request.

The second problem is scenario depth. The `pi-wendao` row currently proves
source retrieval, but its scenarios are still shallow compared with the
docs-corpus SSOT scenarios. It needs multi-hop orchestration questions that
ask for ownership boundaries, workflow-to-host paths, run-state storage, and
prompt/model policy evidence in one answer.

The third problem is intent ambiguity. One source query had to be tightened
from a broad agent-host phrase to an owner-local exported function. That is a
good failure mode because the harness caught ambiguous top-hit behavior, but
it means future receipts need to distinguish owner-definition lookup from
reference-fanout exploration.

## Improvement Direction

The current backend work is profile selection, not another universal search
mode.

1. Keep scenario-aware profile recommendations additive in the Python benchmark
   report. Known-item source lookup can prefer flat/top-k; multi-hop
   knowledge, authority, negative-evidence, and SSOT scenarios should prefer
   graph-first or intent-tree profiles.
2. Add owner-definition versus reference-fanout query diagnostics so AST
   searches can prove whether the user asked for the defining source or the
   surrounding usage graph.
3. Expand external orchestration scenarios for `pi-wendao` so they require
   mixed README, docs, TypeScript source, and workflow evidence rather than
   only isolated hits.
4. Add cross-repository scenario rows where the answer needs Wendao backend
   authority plus external orchestrator ownership evidence.
5. Keep the runtime optimization focus on resident cache/prewarm lifecycle and
   invalidation. The proof harness can remain heavier because it is an
   evidence generator, while the agent request path should reuse warmed
   indexes.
6. Introduce Julia-backed graph profiles only as measured inputs. PPR-like
   relatedness, community grouping, HNSW semantic fanout, and large
   relationship traversal are valuable when receipts expose the relation
   frontier they operate on; they should not become unmeasured defaults.

## Promotion Signals

A future search-profile slice is ready for promotion only when it proves:

- unchanged query and scenario pass counts
- stable path order for asserted top-hit cases
- better evidence coverage or lower exposed context on complex scenarios
- no regression on small known-item scenarios
- explicit profile-choice diagnostics in the benchmark report
- resident/prewarm runtime evidence separate from proof-harness preparation
  time
