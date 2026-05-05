//! External tool runtime integration.

mod bridge;
mod discover_cache;
mod pool;
mod types;

pub use bridge::{
    ToolClientPool, ToolDiscoverCacheStatsSnapshot, ToolListCacheStatsSnapshot,
    ToolPoolConnectConfig,
};
pub use pool::connect_tool_pool;
pub use types::{
    ToolRuntimeCallResult, ToolRuntimeListRequestParams, ToolRuntimeListResult,
    ToolRuntimeToolDefinition,
};
