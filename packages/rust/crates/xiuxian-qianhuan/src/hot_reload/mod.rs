//! Shared hot-reload runtime primitives for Xiuxian modules.
//!
//! This module provides a reusable registration/runtime model that can be
//! mounted by qianhuan, wendao, and future components.

mod backend;
mod driver;
mod policy;
mod runtime;
mod target;

pub use backend::{
    HotReloadTargetId, HotReloadVersionBackend, InMemoryHotReloadVersionBackend,
    ValkeyHotReloadVersionBackend,
};
pub use driver::HotReloadDriver;
pub use policy::{resolve_hot_reload_watch_extensions, resolve_hot_reload_watch_patterns};
pub use runtime::{HotReloadOutcome, HotReloadRuntime, HotReloadStatus, HotReloadTrigger};
pub use target::HotReloadTarget;
