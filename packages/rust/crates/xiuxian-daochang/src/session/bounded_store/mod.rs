//! Bounded session store: `session_id` -> ring buffer of recent turns (`xiuxian-window`).
//! Used when `config.window_max_turns` is set; context for LLM is built from recent turns.

mod snapshot_ops;
mod state;
mod summary_ops;
mod window_ops;

pub use state::{BoundedSessionSnapshotStats, BoundedSessionStats, BoundedSessionStore};
