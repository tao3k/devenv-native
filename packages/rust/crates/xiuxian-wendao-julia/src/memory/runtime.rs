//! Compatibility reexports for memory-family Julia compute runtime bindings.
//!
//! New runtime binding ownership lives in `xiuxian-julia-runtime`.

pub use xiuxian_julia_runtime::wendao::{
    build_memory_julia_compute_binding, build_memory_julia_compute_bindings,
};

#[cfg(test)]
#[path = "../../tests/unit/memory/runtime.rs"]
mod tests;
