//! `search::service::core::repo_runtime` owns Wendao service core repo runtime behavior.

mod bootstrap;
mod helpers;
mod reads;
mod sync;
#[cfg(test)]
#[path = "../../../../../tests/unit/search/service/core/repo_runtime/mod.rs"]
mod tests;
