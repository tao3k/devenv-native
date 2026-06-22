# Wendao Specification: Path-Scoped Code Observation (v3.4)

## 1. Objective

To restrict the search space of code observations to specific files or directories, ensuring high-performance drift detection in Monorepos and preventing cross-package naming collisions.

## 2. Syntax Specification

The `:OBSERVE:` attribute is extended with an optional `scope:` parameter.

### 2.1 Formal Grammar

`:OBSERVE: lang:<lang> [scope:"<glob-path>"] "<sgrep-pattern>"`

### 2.2 Examples

- **Precise File**: `:OBSERVE: lang:rust scope:"packages/xiuxian-wendao/src/lib.rs" "fn process_data"`
- Package Glob : `:OBSERVE: lang:rust scope:"packages/rust/crates/xiuxian-wendao*/src/**/*.rs" "struct Item"`
- **Legacy (Unscoped)**: `:OBSERVE: lang:rust "fn global_helper"` (Deprecated: Fallback to global search)

## 3. Propagation Logic (Sentinel)

When a source file at `SOURCE_PATH` changes:

1. **Filter**: Identify documents where `OBSERVE.scope` matches `SOURCE_PATH`.
2. **Glob Support**: Use standard Unix glob matching if `scope` contains wildcards.
3. **Execution**: Only trigger AST pattern matching if the filter passes.
