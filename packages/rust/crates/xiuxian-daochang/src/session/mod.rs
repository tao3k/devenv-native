//! Session namespace: message types, session store, and optional bounded session store.

mod bounded_store;
mod message;
pub(crate) mod redis_backend;
mod store;
mod summary;

pub use bounded_store::{BoundedSessionSnapshotStats, BoundedSessionStats, BoundedSessionStore};
pub use message::{ChatMessage, FunctionCall, ToolCallOut};
pub(crate) use redis_backend::RedisSessionRuntimeSnapshot;
pub use store::SessionStore;
pub use summary::SessionSummarySegment;
