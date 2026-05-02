//! Canonical path and route constants for Julia link-graph compatibility.

/// Default Julia search launcher path used by Wendao compatibility surfaces.
pub const DEFAULT_JULIA_SEARCH_LAUNCHER_PATH: &str =
    ".data/WendaoSearch.jl/scripts/run_search_service.jl";

/// Default Julia search example config path used by Wendao compatibility surfaces.
pub const DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH: &str =
    ".data/WendaoSearch.jl/config/live/solver_demo.toml";

/// Canonical Arrow Flight rerank route used by Julia compatibility surfaces.
pub const DEFAULT_JULIA_RERANK_FLIGHT_ROUTE: &str = "/rerank";
