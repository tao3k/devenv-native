# Xiuxian Code Intelligence

`xiuxian-code-intelligence` extracts compact code-structure signals for Agent
search. It is the first substrate for turning source files into searchable
symbol outlines and structural matches that Wendao can later index into
reasoning-tree retrieval flows.

## Current Scope

- AST-based symbol extraction through `xiuxian-ast`
- Syntax-aware outlines for Python, Rust, JavaScript, and TypeScript
- Pattern-based structural search for supported languages
- Parser ownership evidence for Agent search and graph projection
- Compact formatted output for Agent context reduction

This crate does not own parsing. `xiuxian-ast` owns parser and ast-grep
compatibility. This crate owns the code-intelligence signal layer built on top
of those parser primitives.

## Usage

```rust
use xiuxian_code_intelligence::CodeIntelligenceExtractor;

let outline = CodeIntelligenceExtractor::outline_file("src/main.rs", Some("rust"))?;
println!("{outline}");
```

`CodeIntelligenceExtractor` is the public entry point for both formatted Agent
context and typed indexing signals.

`CodeParserEvidenceRegistry` is the public entry point for parser ownership
signals such as `parser-priority:*`, `effective-parser:*`, and
`baseline-parser:*` edge tags consumed by Wendao graph/reasoning flows.
`extract_code_structure_symbols` owns reusable skeleton-symbol extraction from
source content.
`score_code_structure_query` owns the reusable code-structure hit scoring
heuristic; Wendao remains responsible for turning scores into repo search hits.

## Typed Signals

Use typed APIs when the caller needs stable data for indexing or ranking:

```rust
use xiuxian_ast::Lang;
use xiuxian_code_intelligence::{
    CodeIntelligenceExtractor, SearchConfig, extract_code_structure_symbols,
};

let symbols = CodeIntelligenceExtractor::outline_file_symbols("src/main.rs", Some("rust"))?;
let structure = extract_code_structure_symbols("pub fn search() {}", Lang::Rust);
let hits = CodeIntelligenceExtractor::search_directory_hits(
    "src",
    "pub fn $NAME",
    SearchConfig::default(),
)?;
```

Formatted `outline_file`, `search_file`, and `search_directory` remain useful
for compact Agent context. Wendao integration should use the typed signal
methods instead of parsing those formatted strings.

Parser evidence should also be consumed through typed APIs:

```rust
use xiuxian_code_intelligence::CodeParserEvidenceRegistry;

let evidence = CodeParserEvidenceRegistry::agent_search_defaults().resolve_path("src/lib.rs");
let edge_kinds = evidence.edge_kinds;
```

## Supported Languages

- Python (`.py`)
- Rust (`.rs`)
- JavaScript (`.js`)
- TypeScript (`.ts`)

## Architecture

See [docs/developer/ast-grep-core.md](../../../../docs/developer/ast-grep-core.md).

## Future Direction

The crate should become useful only when its signals are consumed by Wendao
repo indexing, page-index, link-graph, or reasoning-tree search. If it remains
only a standalone wrapper around `xiuxian-ast`, it should be merged into
`xiuxian-ast` instead of staying as a separate package.
