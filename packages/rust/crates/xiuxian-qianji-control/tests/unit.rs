//! Unit test aggregate for `xiuxian-qianji-control`.

#[path = "unit/control.rs"]
mod control;

#[cfg(feature = "duckdb")]
#[path = "unit/duckdb.rs"]
mod duckdb;

#[cfg(feature = "valkey")]
#[path = "unit/valkey.rs"]
mod valkey;
