#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
mod feature_gate;
mod fixture;
mod openai_compatible;
mod support;
