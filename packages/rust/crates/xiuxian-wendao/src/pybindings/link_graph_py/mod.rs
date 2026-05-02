//! `PyO3` bindings for markdown link graph engine.

mod cache;
#[path = "engine/mod.rs"]
mod engine;

pub use cache::{
    link_graph_stats_cache_del, link_graph_stats_cache_get, link_graph_stats_cache_set,
};
pub use engine::PyLinkGraphEngine;
