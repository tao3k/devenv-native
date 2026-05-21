#[cfg(feature = "duckdb")]
mod cache;
mod catalog;
#[cfg(feature = "julia")]
#[path = "docs_page_index.rs"]
mod docs_page_index;
mod evaluation;
mod run_modes;
#[path = "scenario_matrix.rs"]
mod scenario_matrix;
#[cfg(feature = "julia")]
#[path = "semantic_gate.rs"]
mod semantic_gate;
mod support;
