# xiuxian-ast

> Unified AST Utilities using ast-grep.

## Overview

This crate provides unified AST and structural extraction helpers on top of
ast-grep, plus the Python tree-sitter parser used by the active Rust lanes.

## Features

- Multi-language ast-grep support
- Parser registry for general AST baselines plus local/native/plugin overrides
- Pattern-based code search
- Syntax tree traversal
- Code transformation support
- Structural semantic fingerprints for supported generic AST languages

## Usage

```rust
use xiuxian_ast::{scan, Lang};

let matches = scan("def hello(): pass", "def $NAME", Lang::Python)?;
```

## Supported Languages

- Python
- Rust
- JavaScript/TypeScript
- Go
- Java

Julia and Modelica no longer live in this crate. The active Wendao lane owns
those languages through `WendaoCodeParser.jl` native routes consumed by
`xiuxian-julia-core` over Arrow Flight.

## Parser Registry

`AstParserRegistry` centralizes parser ownership resolution. `xiuxian-ast`
provides the general ast-grep baseline for supported languages. Packages may
register local/native/plugin parsers such as `rust-lang-parser`,
`markdown-lang-parser`, `julia-lang-parser`, and `modelica-lang-parser` as
effective overrides for owned surfaces. The resolver keeps both facts visible:
`effective_parser` records the parser that owns the source, while
`baseline_parser` records the comparable general AST baseline when ast-grep can
parse the same path.

## Testing

- `cargo test -p xiuxian-ast`
- `cargo clippy -p xiuxian-ast --lib --tests -- -D warnings`

## Project Harness Boundary

`xiuxian-ast` uses `rust-lang-project-harness` for project-policy gates. The
source and test gate roots run without disabled rules. Public ast-grep
re-exports are explicit, fingerprinting has an owner doc, and nested unit tests
import package APIs directly so harness facts stay tied to owner modules.

## License

Apache-2.0
