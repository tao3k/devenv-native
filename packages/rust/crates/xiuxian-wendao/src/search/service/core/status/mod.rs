//! `search::service::core::status` owns Wendao service core status behavior.

mod compaction;
mod helpers;
mod repo;
mod runtime;
#[cfg(test)]
#[path = "../../../../../tests/unit/search/service/core/status/mod.rs"]
mod tests;
