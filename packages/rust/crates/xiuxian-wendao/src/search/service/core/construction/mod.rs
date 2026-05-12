//! `search::service::core::construction` owns Wendao service core construction behavior.

mod concurrency;
mod paths;
mod runtime;

#[cfg(test)]
#[path = "../../../../../tests/unit/search/service/core/construction/mod.rs"]
mod tests;
