//! Incremental-first `LinkGraph` refresh mechanism.

#[path = "executors/wendao_refresh/input.rs"]
mod input;
#[path = "executors_wendao_refresh_mechanism.rs"]
mod mechanism;
#[path = "executors/wendao_refresh/refresh.rs"]
mod refresh;

pub use mechanism::WendaoRefreshMechanism;
